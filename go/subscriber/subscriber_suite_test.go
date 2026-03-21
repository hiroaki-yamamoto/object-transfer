package subscriber_test

import (
	"testing"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

func TestSubscriber(t *testing.T) {
	RegisterFailHandler(Fail)
	RunSpecs(t, "Subscriber Suite")
}
