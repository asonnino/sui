// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
// @generated automatically by Diesel CLI.

diesel::table! {
    checkpoints (sequence_number) {
        sequence_number -> Int8,
        checkpoint_digest -> Bytea,
        epoch -> Int8,
        network_total_transactions -> Int8,
        previous_digest -> Bytea,
        end_of_epoch -> Bool,
        transactions -> Array<Nullable<Text>>,
        timestamp_ms -> Int8,
        total_gas_cost -> Int8,
        computation_cost -> Int8,
        storage_cost -> Int8,
        storage_rebate -> Int8,
        non_refundable_storage_fee -> Int8,
        checkpoint_commitments -> Bytea,
        validator_signature -> Bytea,
    }
}

diesel::table! {
    epochs (epoch) {
        epoch -> Int8,
        validators -> Bytea,
        epoch_total_transactions -> Int8,
        first_checkpoint_id -> Int8,
        epoch_start_timestamp -> Int8,
        reference_gas_price -> Int8,
        protocol_version -> Int8,
        end_of_epoch_info -> Nullable<Bytea>,
    }
}

diesel::table! {
    events (tx_sequence_number) {
        tx_sequence_number -> Int8,
        transaction_digest -> Bytea,
        event_sequence -> Int8,
        sender -> Bytea,
        package -> Bytea,
        module -> Text,
        event_type -> Text,
        timestamp_ms -> Int8,
        bcs -> Bytea,
    }
}

diesel::table! {
    objects (object_id) {
        object_id -> Bytea,
        object_version -> Int8,
        owner_type -> Int2,
        owner_id -> Nullable<Bytea>,
        serialized_object -> Array<Nullable<Bytea>>,
        coin_type -> Nullable<Text>,
        coin_balance -> Nullable<Int8>,
        dynamic_field_name_type -> Nullable<Text>,
        dynamic_field_value -> Nullable<Text>,
        dynamic_field_type -> Nullable<Int2>,
    }
}

diesel::table! {
    transactions (tx_sequence_number) {
        tx_sequence_number -> Int8,
        transaction_digest -> Bytea,
        raw_transaction -> Bytea,
        raw_effects -> Bytea,
        checkpoint_sequence_number -> Int8,
        timestamp_ms -> Int8,
        object_changes -> Bytea,
        balance_changes -> Bytea,
        events -> Bytea,
        transaction_kind -> Int2,
    }
}

diesel::table! {
    tx_indices (tx_sequence_number) {
        tx_sequence_number -> Int8,
        transaction_digest -> Bytea,
        input_objects -> Array<Nullable<Bytea>>,
        changed_objects -> Array<Nullable<Bytea>>,
        senders -> Array<Nullable<Bytea>>,
        recipients -> Array<Nullable<Bytea>>,
        package -> Array<Nullable<Bytea>>,
        package_module -> Array<Nullable<Bytea>>,
        package_module_function -> Array<Nullable<Bytea>>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    checkpoints,
    epochs,
    events,
    objects,
    transactions,
    tx_indices,
);
