package subscriber

type Option struct {
	autoAck bool
}

func NewOption() *Option {
	return &Option{
		autoAck: true,
	}
}

func (o *Option) AutoAck(autoAck bool) *Option {
	o.autoAck = autoAck
	return o
}
