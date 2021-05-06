package main

import (
	"archive/zip"
	"bytes"
	"context"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"math/rand"
	"os"
	"strings"

	"github.com/aws/aws-lambda-go/events"
	lambda_handler "github.com/aws/aws-lambda-go/lambda"
	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/lambda"
	"github.com/aws/aws-sdk-go-v2/service/lambda/types"
)

type Response events.APIGatewayV2HTTPResponse
type Request events.APIGatewayV2HTTPRequest

type Parameters struct {
	SourceCode string `json:"sourceCode"`
}

var HEADERS = map[string]string{
	"Content-Type": "text/plain",
	"X-Powered-By": "Sadness, mostly",
}

// Handler is our lambda handler invoked by the `lambda.Start` function call
func Handler(ctx context.Context, request Request) (Response, error) {
	fmt.Println(request.RequestContext.HTTP.Method, " ", request.RawPath, " ", request.Body)
	switch request.RequestContext.HTTP.Method {
	case "POST":
		return postHandler(request)
	case "GET":
		fallthrough
	default:
		return getHandler(request)
	}
}

func getHandler(request Request) (Response, error) {
	params := Parameters{}
	resp := Response{}

	if value, ok := request.QueryStringParameters["sourceCode"]; ok {
		params.SourceCode = value

		invoke_output, err := invokeLambda(params.SourceCode)
		if err != nil {
			return Response{
				StatusCode: 500,
				Body:       fmt.Sprintf("Error when building lambda: %s", err.Error()),
				Headers:    HEADERS,
			}, nil
		}

		output_string := string(invoke_output.Payload)
		// Just hope and pray this lambda never runs "warm"
		HEADERS["Content-Type"] = "application/json"
		resp = Response{
			StatusCode: 200,
			Body:       output_string,
			Headers:    HEADERS,
		}
	} else {
		resp = Response{
			StatusCode: 400,
			Body:       "You didn't supply sourceCode!",
			Headers:    HEADERS,
		}
	}

	return resp, nil
}

func postHandler(request Request) (Response, error) {
	params := Parameters{}
	if request.IsBase64Encoded {
		if value, err := base64.StdEncoding.DecodeString(request.Body); err != nil {
			return Response{
				StatusCode: 400,
				Body:       fmt.Sprintf("Error parsing base64-encoded parameters: %s", err.Error()),
				Headers:    HEADERS,
			}, nil
		} else {
			request.Body = string(value)
		}
	}
	fmt.Println("JSON ", request.IsBase64Encoded, " ", request.Body)
	buf := []byte(request.Body)
	if err := json.Unmarshal(buf, &params); err != nil {
		return Response{
			StatusCode: 400,
			Body:       "Error parsing JSON body!",
			Headers:    HEADERS,
		}, nil
	} else {

		return Response{
			StatusCode: 200,
			Body:       params.SourceCode,
			Headers:    HEADERS,
		}, nil
	}
}

func invokeLambda(sourceCode string) (lambda.InvokeOutput, error) {
	name_channel := make(chan string)
	code_channel := make(chan []byte)
	role_channel := make(chan string)

	go makeRandom(16, name_channel)
	go getRole(role_channel)
	go makeCode(sourceCode, code_channel)

	config, _ := config.LoadDefaultConfig(context.TODO())
	client := lambda.NewFromConfig(config)

	function_name := <-name_channel
	create_input := &lambda.CreateFunctionInput{
		Code: &types.FunctionCode{
			ZipFile: <-code_channel,
		},
		Description:  aws.String("Invoke the provided codez"),
		FunctionName: aws.String(function_name),
		Handler:      aws.String("handler.main"),
		Role:         aws.String(<-role_channel),
		Runtime:      types.RuntimePython38,
	}
	log.Println("Creating function ", function_name)
	_, err := client.CreateFunction(context.TODO(), create_input)
	if err != nil {
		log.Println(err.Error())
		return lambda.InvokeOutput{}, errors.New("Could not create lambda function")
	}

	invoke_input := &lambda.InvokeInput{
		FunctionName: aws.String(function_name),
	}
	invoke_output, err := client.Invoke(context.TODO(), invoke_input)
	if err != nil {
		log.Println(err.Error())
		return lambda.InvokeOutput{}, errors.New("Could not invoke lambda function")
	}

	go func() {
		client.DeleteFunction(context.TODO(), &lambda.DeleteFunctionInput{
			FunctionName: aws.String(function_name),
		})
	}()

	return *invoke_output, nil
}

var LETTERS = []rune("abcdefghijklmnopqrstuvwxyz")

// Returns a random string of lowercase alphabetical characters of specified length
func makeRandom(length int, output_channel chan string) {
	log.Println("Making random function name...")
	random_runes := make([]rune, length)
	for i := range random_runes {
		random_runes[i] = LETTERS[rand.Intn(len(LETTERS))]
	}
	log.Println("Function name generated: ", string(random_runes))
	output_channel <- string(random_runes)
}

// Given a raw python string, returns a Lambda-compatible bytestream with the code in `handler.py`
// Which is to say - make a zipfile with the only file being `handler.py` with the contents
func makeCode(input_code string, output_channel chan []byte) {
	log.Println("Creating the Zip for the Lambda...")
	buf := new(bytes.Buffer)
	writer := zip.NewWriter(buf)

	f, err := writer.Create("handler.py")
	if err != nil {
		log.Fatal(err)
	}

	// Add the function definition and tab everything out
	final_code := "def main(event, context):\n"
	split_code := strings.Split(input_code, "\n")
	for line := range split_code {
		final_code += fmt.Sprintf("\t%s\n", split_code[line])
	}

	if _, err := f.Write([]byte(final_code)); err != nil {
		log.Fatal(err)
	}
	writer.Close()
	log.Println("Zip generated")
	output_channel <- buf.Bytes()
}

// Collect the role ARN of our own function, to then use it to create another function
func getRole(output_channel chan string) {
	log.Println("Getting the role ARN for the current function")
	config, _ := config.LoadDefaultConfig(context.TODO())
	client := lambda.NewFromConfig(config)
	func_name := os.Getenv("AWS_LAMBDA_FUNCTION_NAME")

	result, err := client.GetFunction(context.TODO(), &lambda.GetFunctionInput{
		FunctionName: aws.String(func_name),
	})
	if err != nil {
		log.Fatal(err)
	}
	log.Println("Got role ", *result.Configuration.Role)
	output_channel <- *result.Configuration.Role
}

func main() {
	lambda_handler.Start(Handler)
}
