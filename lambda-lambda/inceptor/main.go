package main

import (
	"context"

	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
)

type Response events.APIGatewayProxyResponse
type Request events.APIGatewayProxyRequest

type Parameters struct {
	SourceCode string `json:"sourceCode"`
}

var HEADERS = map[string]string{
	"Content-Type": "text/plain",
	"X-Powered-By": "Sadness, mostly",
}

// Handler is our lambda handler invoked by the `lambda.Start` function call
func Handler(ctx context.Context, request Request) (Response, error) {
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

func main() {
	lambda.Start(Handler)
}
