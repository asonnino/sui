CREATE TABLE tx_indices (
    tx_sequence_number          BIGINT       PRIMARY KEY,
    transaction_digest          bytea        NOT NULL,
    input_objects               bytea[]      NOT NULL,
    changed_objects             bytea[]      NOT NULL,
    senders                     bytea[]      NOT NULL,
    recipients                  bytea[]      NOT NULL,
    package                     bytea[]      NOT NULL,
    package_module              bytea[]      NOT NULL,
    package_module_function     bytea[]      NOT NULL
);

CREATE INDEX tx_indices_input_objects ON tx_indices USING GIN(input_objects);
CREATE INDEX tx_indices_chagned_objects ON tx_indices USING GIN(changed_objects);
CREATE INDEX tx_indices_senders ON tx_indices USING GIN(senders);
CREATE INDEX tx_indices_recipients ON tx_indices USING GIN(recipients);
CREATE INDEX tx_indices_package ON tx_indices USING GIN(package);
CREATE INDEX tx_indices_package_module ON tx_indices USING GIN(package_module);
CREATE INDEX tx_indices_package_module_function ON tx_indices USING GIN(package_module_function);
