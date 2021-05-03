package main

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"

	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
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

		resp = Response{
			StatusCode: 200,
			Body:       params.SourceCode,
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

func main() {
	lambda.Start(Handler)
}
