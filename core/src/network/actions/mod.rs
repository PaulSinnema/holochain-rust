pub mod custom_send;
pub mod query;
pub mod get_validation_package;
pub mod initialize_network;
pub mod publish;
pub mod shutdown;
pub mod publish_header_entry;

use holochain_core_types::error::HcResult;
use holochain_persistence_api::cas::content::Address;

#[derive(Clone, Debug)]
pub enum ActionResponse {
    Publish(HcResult<Address>),
    PublishHeaderEntry(HcResult<Address>),
    Respond(HcResult<()>),
}
