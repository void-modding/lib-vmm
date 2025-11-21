use crate::{capabilities::{api_key_capability::{ApiKeyCapability, KeyAction, RequiresApiKey}, ids}, tests::dummy::DummyModProvider, traits::provider::Provider};

#[test]
fn api_key_cap_validates() {
    let provider = DummyModProvider::new("dummy");
    let cap = provider.capabilities()
        .iter()
        .find(|o| o.id() == ids::REQUIRES_API_KEY)
        .expect("Api key cap missing");

    let api_cap = cap.as_any()
        .downcast_ref::<ApiKeyCapability<DummyModProvider>>()
        .expect("wrong capability type");

    assert!(api_cap.needs_prompt(None));
    let result = api_cap.on_provided("ABCDEFGHIJKLMNOP");
    assert!(matches!(result, Ok(KeyAction::Store)))
}
