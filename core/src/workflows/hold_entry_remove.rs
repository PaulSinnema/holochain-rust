use crate::{
    context::Context, dht::actions::remove_entry::remove_entry,
    network::entry_with_header::EntryWithHeader, nucleus::validation::validate_entry,
};

use crate::{
    nucleus::{
        actions::add_pending_validation::add_pending_validation, validation::ValidationError,
    },
    scheduled_jobs::pending_validations::ValidatingWorkflow,
    workflows::validation_package,
};
use holochain_core_types::{
    entry::Entry,
    error::HolochainError,
    validation::{EntryLifecycle, ValidationData},
};
use holochain_persistence_api::cas::content::AddressableContent;
use std::sync::Arc;

pub async fn hold_remove_workflow(
    entry_with_header: &EntryWithHeader,
    context: Arc<Context>,
) -> Result<(), HolochainError> {
    // 1. Get hold of validation package
    let await_package = await!(validation_package(entry_with_header, context.clone()));
    let maybe_validation_package = await_package.map_err(|err| {
        let message = "Could not get validation package from source! -> Add to pending...";
        context.log(format!("debug/workflow/hold_remove: {}", message));
        context.log(format!("debug/workflow/hold_remove: Error was: {:?}", err));
        add_pending_validation(
            entry_with_header.to_owned(),
            Vec::new(),
            ValidatingWorkflow::RemoveEntry,
            context.clone(),
        );
        HolochainError::ValidationPending
    })?;
    let validation_package = maybe_validation_package
        .ok_or("Could not get validation package from source".to_string())?;

    // 2. Create validation data struct
    let validation_data = ValidationData {
        package: validation_package,
        lifecycle: EntryLifecycle::Meta,
    };

    // 3. Validate the entry
    let await_validate = await!(validate_entry(
        entry_with_header.entry.clone(),
        None,
        validation_data,
        &context,
    ));
    await_validate
    .map_err(|err| {
        if let ValidationError::UnresolvedDependencies(dependencies) = &err {
            context.log(format!("debug/workflow/hold_remove: Entry removal could not be validated due to unresolved dependencies and will be tried later. List of missing dependencies: {:?}", dependencies));
            add_pending_validation(
                entry_with_header.to_owned(),
                dependencies.clone(),
                ValidatingWorkflow::RemoveEntry,
                context.clone(),
            );
            HolochainError::ValidationPending
        } else {
            context.log(format!(
                "info/workflow/hold_remove: Entry removal {:?} is NOT valid! Validation error: {:?}",
                entry_with_header.entry,
                err,
            ));
            HolochainError::from(err)
        }

    })?;

    let deletion_entry = unwrap_to!(entry_with_header.entry => Entry::Deletion);

    let deleted_entry_address = deletion_entry.clone().deleted_entry_address();
    // 3. If valid store the entry in the local DHT shard
    await!(remove_entry(
        &context.clone(),
        deleted_entry_address,
        entry_with_header.entry.address().clone(),
    ))
}
