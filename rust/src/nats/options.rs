use ::std::sync::Arc;

use ::async_nats::jetstream::{
  consumer::pull::Config as PullConfig, stream::Config as StreamConfig,
};

use crate::r#enum::Format;

#[derive(Debug)]
pub struct AckSubOptions {
  pub(super) stream_cfg: StreamConfig,
  pub(super) pull_cfg: PullConfig,
  pub(super) auto_ack: bool,
  pub(super) format: Format,
}

impl AckSubOptions {
  pub fn new(format: Format, name: Arc<str>) -> Self {
    Self {
      stream_cfg: StreamConfig {
        name: name.clone().to_string(),
        ..Default::default()
      },
      pull_cfg: PullConfig {
        name: Some(name.clone().to_string()),
        ..PullConfig::default()
      },
      auto_ack: true,
      format,
    }
  }

  pub fn auto_ack(mut self, auto_ack: bool) -> Self {
    self.auto_ack = auto_ack;
    return self;
  }

  pub fn name(mut self, name: impl Into<String>) -> Self {
    self.stream_cfg.name = name.into();
    self
  }

  pub fn subjects(mut self, subjects: Vec<impl Into<String>>) -> Self {
    self.stream_cfg.subjects = subjects.into_iter().map(Into::into).collect();
    self
  }

  pub fn durable_name(mut self, durable_name: impl Into<String>) -> Self {
    self.pull_cfg.durable_name = Some(durable_name.into());
    self
  }

  pub fn format(mut self, format: Format) -> Self {
    self.format = format;
    self
  }

  pub fn stream_config(mut self, stream_cfg: StreamConfig) -> Self {
    self.stream_cfg = stream_cfg;
    self
  }

  pub fn pull_config(mut self, pull_cfg: PullConfig) -> Self {
    self.pull_cfg = pull_cfg;
    self
  }
}
