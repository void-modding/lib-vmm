use std::sync::Arc;

use crate::{
    capabilities::{
        api_key_capability::{
            ApiKeyCapability, ApiKeyValidationError, KeyAction, RequiresApiKey, ApiSubmitResponse,
        },
        base::{Capability, CapabilityCastExt, CapabilityRef},
        builder::CapabilityBuilder,
        ids,
    },
    capability,
    tests::dummy::DummyModProvider,
    traits::provider::Provider,
};

#[test]
fn api_key_cap_validates() {
    let provider = DummyModProvider::new("dummy");
    let cap = provider
        .capabilities()
        .iter()
        .find(|o| o.id() == ids::REQUIRES_API_KEY)
        .expect("Api key cap missing");

    let api_cap = cap
        .as_any()
        .downcast_ref::<ApiKeyCapability<DummyModProvider>>()
        .expect("wrong capability type");

    assert!(api_cap.needs_prompt(None));
    let schema = api_cap.render().expect("form schema should exist");
    let resp = ApiSubmitResponse {
        id: schema.fields[0].id.clone(),
        value: "ABCDEFGHIJKLMNOP".to_string(),
    };
    let responses = vec![resp];
    let result = api_cap.on_provided(&responses);
    assert!(matches!(result, Ok(KeyAction::Store)))
}

#[test]
fn api_key_cap_error_cases() {
    let provider = DummyModProvider::new("dummy");
    let cap = provider
        .capabilities()
        .iter()
        .find(|o| o.id() == ids::REQUIRES_API_KEY)
        .unwrap();
    let api_cap = cap
        .as_any()
        .downcast_ref::<ApiKeyCapability<DummyModProvider>>()
        .unwrap();

    let schema = api_cap.render().expect("form schema should exist");

    // Empty string
    let resp_empty = ApiSubmitResponse {
        id: schema.fields[0].id.clone(),
        value: "".to_string(),
    };
    let responses_empty = vec![resp_empty];
    assert!(matches!(
        api_cap.on_provided(&responses_empty),
        Err(ApiKeyValidationError::Empty)
    ));

    // Too short
    let resp_short = ApiSubmitResponse {
        id: schema.fields[0].id.clone(),
        value: "SHORT".to_string(),
    };
    let responses_short = vec![resp_short];
    assert!(matches!(
        api_cap.on_provided(&responses_short),
        Err(ApiKeyValidationError::TooShort { min_len: 16 })
    ));

    // Valid
    let resp_valid = ApiSubmitResponse {
        id: schema.fields[0].id.clone(),
        value: "ABCDEFGHIJKLMNOP".to_string(),
    };
    let responses_valid = vec![resp_valid];
    assert!(matches!(
        api_cap.on_provided(&responses_valid),
        Ok(KeyAction::Store)
    ));
}

#[test]
#[should_panic(expected = "form schema should exist: ProviderDropped")]
fn api_key_cap_provider_dropped_behaviors() {
    let cap: CapabilityRef = {
        let provider = DummyModProvider::new("dummy");
        provider.capabilities()[0].clone()
    };

    let api_cap = cap
        .as_any()
        .downcast_ref::<ApiKeyCapability<DummyModProvider>>()
        .unwrap();

    let schema = api_cap.render().expect("form schema should exist");
    let resp = ApiSubmitResponse {
        id: schema.fields[0].id.clone(),
        value: "ABCDEFGHIJKLMNOP".to_string(),
    };
    let responses = vec![resp];

    // Provider dropped: on_provided should panic (not return ProviderError)
    let _ = api_cap.on_provided(&responses);

}

#[test]
fn api_key_cap_provider_dropped_render_errors() {
    let cap: CapabilityRef = {
        let provider = DummyModProvider::new("dummy");
        provider.capabilities()[0].clone()
    };

    let api_cap = cap
        .as_any()
        .downcast_ref::<ApiKeyCapability<DummyModProvider>>()
        .unwrap();

    let res = api_cap.render();
    assert!(res.is_err());

}

#[test]
fn capability_cast_ext_helper() {
    let provider = DummyModProvider::new("dummy");
    let cap = provider.capabilities()[0].clone();
    let dyn_ref: &dyn Capability = &*cap;
    let typed = dyn_ref.get::<ApiKeyCapability<DummyModProvider>>();
    assert!(typed.is_some());
    assert_eq!(typed.unwrap().id(), ids::REQUIRES_API_KEY);
}

#[test]
fn capability_builder_api_key_chain() {
    let provider = DummyModProvider::new("builder-test");
    let caps = CapabilityBuilder::new_from_arc(&provider)
        .api_key()
        .finish();

    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].id(), ids::REQUIRES_API_KEY);
}

struct SimpleCap;
capability!(SimpleCap, "test.simple");

#[test]
fn capability_macro_assigns_id_and_downcast() {
    let cap: CapabilityRef = Arc::new(SimpleCap);
    assert_eq!(cap.id(), "test.simple");
    let dyn_ref: &dyn Capability = &*cap;
    assert!(dyn_ref.get::<SimpleCap>().is_some());
}
