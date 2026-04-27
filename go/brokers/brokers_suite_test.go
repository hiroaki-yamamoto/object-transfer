package brokers_test

import (
	"testing"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
)

func TestBrokers(t *testing.T) {
	RegisterFailHandler(Fail)
	RunSpecs(t, "Brokers Suite")
}
