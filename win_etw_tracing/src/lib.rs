// Copyright (C) Microsoft Corporation. All rights reserved.

//! Subscriber for tracing events that emits Windows ETW tracelogging events.
#![cfg(windows)]
#![forbid(unsafe_code)]

use bytes::BufMut;
use core::fmt;
use std::io::Write;
use tracing::field::Field;
use tracing::field::Visit;
use tracing::span::Attributes;
use tracing::span::Record;
use tracing::Event;
use tracing::Id;
use tracing::Metadata;
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use win_etw_metadata::InFlag;
use win_etw_metadata::OutFlag;
use win_etw_provider::Error;
use win_etw_provider::EtwProvider;
use win_etw_provider::EventDataDescriptor;
use win_etw_provider::EventDescriptor;
use win_etw_provider::EventOptions;
use win_etw_provider::Provider;
use win_etw_provider::GUID;

/// An implementation for [`tracing_subscriber::Layer`] that emits tracelogging
/// events.
pub struct TracelogSubscriber {
    provider: EtwProvider,
    keyword_mask: u64,
    global_fields: EventData,
    trace_keyword: u64,
}

impl TracelogSubscriber {
    /// Creates a new subscriber with provider ID `id` and provider name `name`.
    pub fn new(id: impl Into<GUID>, name: &str) -> Result<Self, Error> {
        let mut provider_metadata = Vec::new();
        provider_metadata.put_u16_le(
            (2 + name.len() + 1)
                .try_into()
                .expect("provider name too long"),
        );
        provider_metadata.put_slice(name.as_bytes());
        provider_metadata.put_u8(0);

        let mut provider = EtwProvider::new(&id.into())?;
        provider.register_provider_metadata(provider_metadata.as_slice())?;
        Ok(Self {
            provider,
            keyword_mask: !0_u64,
            global_fields: EventData {
                metadata: Vec::new(),
                data: Vec::new(),
            },
            trace_keyword: 0,
        })
    }

    // If some events are by default marked with telemetry keywords, this allows an opt out.
    pub fn enable_telemetry_events(&mut self, enabled: bool) {
        self.keyword_mask = if enabled {
            !0_u64
        } else {
            !(win_etw_metadata::MICROSOFT_KEYWORD_CRITICAL_DATA
                | win_etw_metadata::MICROSOFT_KEYWORD_MEASURES
                | win_etw_metadata::MICROSOFT_KEYWORD_TELEMETRY)
        };
    }

    pub fn filter_keyword(&self, keyword: u64) -> u64 {
        keyword & self.keyword_mask
    }

    /// Global fields are automatically included in all events emitted by this
    /// layer. They can be set at the time of layer creation, or by using
    /// [`tracing_subscriber::reload`] to dynamically reconfigure a registered
    /// layer. Note that if the subscriber is registered as the [global
    /// default](tracing::dispatcher#setting-the-default-subscriber), thesee
    /// fields will be global to the entire process.
    ///
    /// # Example
    /// ```
    /// # use win_etw_tracing::TracelogSubscriber;
    /// # use win_etw_provider::GUID;
    /// # let provider_guid = GUID {
    /// #     data1: 0xe1c71d95,
    /// #     data2: 0x7bbc,
    /// #     data3: 0x5f48,
    /// #     data4: [0xa9, 0x2b, 0x8a, 0xaa, 0x0b, 0x52, 0x91, 0x58],
    /// # };
    /// let mut layer = TracelogSubscriber::new(provider_guid, "provider_name").unwrap();
    /// let globals = vec![("field name", "my value")];
    /// layer.set_global_fields(&globals);
    /// ```
    pub fn set_global_fields(&mut self, fields: &[(&str, &str)]) {
        self.global_fields.metadata.clear();
        self.global_fields.data.clear();
        for &(name, value) in fields.iter() {
            self.global_fields.record_global(name, value);
        }
    }

    /// Sets the keyword to use for events logged at [`tracing::Level::TRACE`]
    /// level.
    ///
    /// Because ETW only provides one level below [`win_etw_metadata::Level::INFO`],
    /// both [`tracing::Level::DEBUG`] and [`tracing::Level::TRACE`] events are
    /// mapped to [`win_etw_metadata::Level::VERBOSE`]. This method allows
    /// distinguishing between the two levels by assigning a specific keyword
    /// used only for [`tracing::Level::TRACE`] events.
    ///
    /// By default, this is set to `0`, meaning no keyword is applied.
    pub fn set_trace_keyword(&mut self, keyword: u64) {
        self.trace_keyword = keyword;
    }
}

impl TracelogSubscriber {
    fn write_event(
        &self,
        opcode: u8,
        options: &EventOptions,
        write_target: bool,
        meta: &Metadata<'_>,
        write_name: impl FnOnce(&mut Vec<u8>),
        record: impl FnOnce(&mut dyn Visit),
    ) {
        let mut keyword = 0;
        let level = match *meta.level() {
            tracing::Level::ERROR => win_etw_metadata::Level::ERROR,
            tracing::Level::WARN => win_etw_metadata::Level::WARN,
            tracing::Level::INFO => win_etw_metadata::Level::INFO,
            tracing::Level::DEBUG => win_etw_metadata::Level::VERBOSE,
            tracing::Level::TRACE => {
                keyword = self.trace_keyword;
                win_etw_metadata::Level::VERBOSE
            }
        };

        let event_descriptor = EventDescriptor {
            id: 0,
            version: 0,
            channel: 11, // this value tells older versions of ETW that this is a tracelogging event
            level,
            opcode,
            task: 0,
            keyword: self.filter_keyword(keyword),
        };

        if !self.provider.is_event_enabled(&event_descriptor) {
            return;
        }

        let mut event_data = EventData {
            metadata: Vec::new(),
            data: Vec::new(),
        };
        event_data.metadata.put_u16_le(0); // reserve space for the size
        event_data.metadata.put_u8(0); // no extensions
        write_name(&mut event_data.metadata);
        event_data.metadata.put_u8(0); // null terminator

        let target_len = if write_target {
            // Target field
            event_data.metadata.put_slice(b"target\0");
            event_data
                .metadata
                .put_u8((InFlag::COUNTED_ANSI_STRING | InFlag::CHAIN_FLAG).bits());
            event_data.metadata.put_u8(OutFlag::UTF8.bits());
            meta.target().len() as u16
        } else {
            0
        };

        event_data
            .metadata
            .put_slice(self.global_fields.metadata.as_slice());
        event_data
            .data
            .put_slice(self.global_fields.data.as_slice());
        record(&mut event_data);

        // Update the length.
        let event_metadata_len = event_data.metadata.len() as u16;
        (&mut event_data.metadata[0..2]).put_u16_le(event_metadata_len);

        // N.B. Since we pre-registered the provider information when creating
        // the provider, there is no need to log it again here.
        let (data_descriptors_with_target, data_descriptors_without_target);
        let data_descriptors = if write_target {
            data_descriptors_with_target = [
                EventDataDescriptor::for_event_metadata(event_data.metadata.as_slice()),
                EventDataDescriptor::from(&target_len),
                EventDataDescriptor::from(meta.target()),
                EventDataDescriptor::for_bytes(&event_data.data),
            ];
            &data_descriptors_with_target[..]
        } else {
            data_descriptors_without_target = [
                EventDataDescriptor::for_event_metadata(event_data.metadata.as_slice()),
                EventDataDescriptor::for_bytes(&event_data.data),
            ];
            &data_descriptors_without_target[..]
        };
        self.provider
            .write(Some(options), &event_descriptor, data_descriptors);
    }
}

#[derive(Debug, Clone, Default)]
struct ActivityId(GUID);

impl ActivityId {
    #[allow(dead_code)]
    fn new() -> Result<Self, Error> {
        Ok(Self(win_etw_provider::new_activity_id()?))
    }

    fn from_current_thread() -> Result<Self, Error> {
        Ok(Self(win_etw_provider::get_current_thread_activity_id()?))
    }
}

const WINEVENT_OPCODE_INFO: u8 = 0;
const WINEVENT_OPCODE_START: u8 = 1;
const WINEVENT_OPCODE_STOP: u8 = 2;

impl<S: Subscriber> Layer<S> for TracelogSubscriber
where
    S: for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let activity_id = ActivityId::from_current_thread().unwrap_or_default();

        let related_activity_id = {
            if attrs.is_contextual() {
                ctx.current_span().id().cloned()
            } else {
                attrs.parent().cloned()
            }
            .and_then(|id| {
                ctx.span(&id)
                    .unwrap()
                    .extensions()
                    .get::<ActivityId>()
                    .cloned()
            })
            .map(|x| x.0)
        };

        // Store the activity ID on the span to look up later.
        ctx.span(id)
            .unwrap()
            .extensions_mut()
            .insert(activity_id.clone());

        self.write_event(
            WINEVENT_OPCODE_START,
            &EventOptions {
                activity_id: Some(activity_id.0),
                related_activity_id,
                ..Default::default()
            },
            true,
            attrs.metadata(),
            |metadata| metadata.extend(attrs.metadata().name().as_bytes()),
            |visit| attrs.record(visit),
        );
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        // Defer the recorded value until on_close is called. Ideally we would
        // just log the additional data as another event and the data would be
        // aggregated with the rest of the activity's data, but WPA and other
        // analysis tools don't actually handle this.
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();
        let deferred = if let Some(deferred) = extensions.get_mut::<DeferredValues>() {
            deferred
        } else {
            extensions.insert(DeferredValues::default());
            extensions.get_mut().unwrap()
        };
        values.record(deferred);
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        #[cfg(feature = "tracing-log")]
        let normalized_meta = tracing_log::NormalizeEvent::normalized_metadata(event);
        #[cfg(feature = "tracing-log")]
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        #[cfg(not(feature = "tracing-log"))]
        let meta = event.metadata();

        let activity_id = ctx
            .event_span(event)
            .and_then(|span| span.extensions().get::<ActivityId>().cloned().map(|x| x.0));

        self.write_event(
            WINEVENT_OPCODE_INFO,
            &EventOptions {
                activity_id,
                ..Default::default()
            },
            true,
            meta,
            // Write the message as the event name. This will not be ideal for
            // events with dynamic names, but it should work well for structured
            // events, and it follows the precedent set by the tracing-opentelemetry
            // crate.
            |metadata| event.record(&mut EventName(metadata)),
            |visit| event.record(visit),
        );
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).unwrap();
        let extensions = span.extensions();
        let ActivityId(activity_id) = extensions.get::<ActivityId>().cloned().unwrap();
        let values = extensions.get::<DeferredValues>();
        self.write_event(
            WINEVENT_OPCODE_STOP,
            &EventOptions {
                activity_id: Some(activity_id),
                ..Default::default()
            },
            false,
            span.metadata(),
            |metadata| metadata.extend(span.metadata().name().as_bytes()),
            |visit| {
                if let Some(values) = values {
                    values.record(visit)
                };
            },
        );
    }
}

/// Collection of deferred values to log when the span is closed.
#[derive(Default)]
struct DeferredValues {
    values: Vec<(Field, DeferredValue)>,
}

impl DeferredValues {
    fn update(&mut self, field: &Field, value: DeferredValue) {
        for (f, v) in &mut self.values {
            if f == field {
                *v = value;
                return;
            }
        }
        self.values.push((field.clone(), value));
    }

    fn record(&self, visit: &mut dyn Visit) {
        for (field, v) in &self.values {
            match v {
                DeferredValue::Unsigned(v) => visit.record_u64(field, *v),
                DeferredValue::Signed(v) => visit.record_i64(field, *v),
                DeferredValue::Boolean(v) => visit.record_bool(field, *v),
                DeferredValue::String(v) => visit.record_str(field, v),
            }
        }
    }
}

impl Visit for DeferredValues {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.update(field, DeferredValue::String(format!("{value:?}")));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.update(field, DeferredValue::Signed(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.update(field, DeferredValue::Unsigned(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.update(field, DeferredValue::Boolean(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.update(field, DeferredValue::String(value.to_string()));
    }
}

enum DeferredValue {
    Unsigned(u64),
    Signed(i64),
    Boolean(bool),
    String(String),
}

struct EventName<'a>(&'a mut Vec<u8>);

impl Visit for EventName<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.0, "{value:?}");
        }
    }
}

struct EventData {
    metadata: Vec<u8>,
    data: Vec<u8>,
}

impl EventData {
    fn write_name(&mut self, name: &str) -> bool {
        // Skip the message (used as the event name) as well as any log crate
        // metadata (already consumed).
        if name == "message" || (cfg!(feature = "tracing-log") && name.starts_with("log.")) {
            return false;
        }
        self.metadata.put_slice(name.as_bytes());
        self.metadata.put_u8(0); // null terminator
        true
    }

    fn record_global(&mut self, name: &str, value: &str) {
        if self.write_name(name) {
            self.metadata
                .put_u8((InFlag::ANSI_STRING | InFlag::CHAIN_FLAG).bits());
            self.metadata.put_u8(OutFlag::UTF8.bits());
            self.data.extend(value.as_bytes());
            self.data.put_u8(0); // null terminator
        }
    }
}

impl Visit for EventData {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if self.write_name(field.name()) {
            self.metadata
                .put_u8((InFlag::ANSI_STRING | InFlag::CHAIN_FLAG).bits());
            self.metadata.put_u8(OutFlag::UTF8.bits());
            let _ = write!(&mut self.data, "{value:?}\0");
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if self.write_name(field.name()) {
            self.metadata.put_u8(InFlag::INT64.bits());
            self.data.put_i64_le(value);
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        if self.write_name(field.name()) {
            self.metadata
                .put_u8((InFlag::UINT64 | InFlag::CHAIN_FLAG).bits());
            self.metadata.put_u8(OutFlag::HEX.bits());
            self.data.put_u64_le(value);
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        if self.write_name(field.name()) {
            self.metadata.put_u8(InFlag::UINT8.bits());
            self.metadata.put_u8(OutFlag::BOOLEAN.bits());
            self.data.put_u8(value.into());
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if self.write_name(field.name()) {
            self.metadata
                .put_u8((InFlag::ANSI_STRING | InFlag::CHAIN_FLAG).bits());
            self.metadata.put_u8(OutFlag::UTF8.bits());
            self.data.extend(value.as_bytes());
            self.data.put_u8(0); // null terminator
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        if self.write_name(field.name()) {
            self.metadata
                .put_u8((InFlag::ANSI_STRING | InFlag::CHAIN_FLAG).bits());
            self.metadata.put_u8(OutFlag::UTF8.bits());
            let _ = write!(&mut self.data, "{value}");
            let mut source = value.source();
            while let Some(v) = source.take() {
                let _ = write!(&mut self.data, ": {v}");
                source = v.source();
            }
            self.data.put_u8(0); // null terminator
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TracelogSubscriber;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::reload;
    use tracing_subscriber::Registry;
    use win_etw_provider::GUID;

    static PROVIDER_GUID: GUID = GUID {
        data1: 0xe1c71d95,
        data2: 0x7bbc,
        data3: 0x5f48,
        data4: [0xa9, 0x2b, 0x8a, 0xaa, 0x0b, 0x52, 0x91, 0x58],
    };

    static PROVIDER_NAME: &str = "rust-test-provider";

    #[test]
    fn basic() {
        let layer = TracelogSubscriber::new(PROVIDER_GUID.clone(), PROVIDER_NAME).unwrap();
        let _x = Registry::default().with(layer).set_default();
        tracing::info!(foo = 123, bar = 456, "hi {baz}", baz = "what");
        tracing::error!(foo = true, bar = ?PROVIDER_GUID);
        let err = anyhow::anyhow!("failed")
            .context("really failed")
            .context("this thing failed");
        tracing::error!(error = &*err as &dyn std::error::Error, "disaster");
    }

    #[test]
    fn span() {
        let layer = TracelogSubscriber::new(PROVIDER_GUID.clone(), PROVIDER_NAME).unwrap();
        let _x = Registry::default().with(layer).set_default();
        tracing::info_span!("geo", bar = 456).in_scope(|| {
            let span = tracing::info_span!("dude", baz = 789, later = tracing::field::Empty);
            span.in_scope(|| {
                tracing::info!("test");
                span.record("later", true);
                span.record("later", "wait no it's a string now");
            });
        });
    }

    #[test]
    fn global() {
        let (layer, reload_handle) = reload::Layer::new(
            TracelogSubscriber::new(PROVIDER_GUID.clone(), PROVIDER_NAME).unwrap(),
        );
        let _x = Registry::default().with(layer).set_default();
        tracing::info!(a_field = 123, "test globals");
        let global = vec![("global", "some value")];
        reload_handle
            .modify(|layer| layer.set_global_fields(&global))
            .unwrap();
        tracing::info!(a_field = 456, "test globals modify");
        let _s = tracing::info_span!("span with globals", span_field = "abc").entered();
        let global = vec![("global", "new value"), ("global2", "value")];
        reload_handle
            .modify(|layer| layer.set_global_fields(&global))
            .unwrap();
        tracing::info!(a_field = 789, "test globals modify again");
    }
}
