CREATE TABLE events
(
    tx_sequence_number          BIGINT       PRIMARY KEY,
    transaction_digest          bytea        NOT NULL,
    event_sequence              BIGINT       NOT NULL,
    sender                      bytea        NOT NULL,
    package                     bytea        NOT NULL,
    module                      text         NOT NULL,
    -- type_ in SuiEvent::MoveEvent
    event_type                  text         NOT NULL,
    timestamp_ms                BIGINT       NOT NULL,
    bcs                         BYTEA        NOT NULL
);

CREATE INDEX events_sender ON events USING HASH (sender);
CREATE INDEX events_package_module ON events (package, module);
CREATE INDEX events_event_type ON events USING HASH (event_type);
