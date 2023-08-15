CREATE TABLE objects (
    object_id                   bytea         PRIMARY KEY,
    object_version              bigint        NOT NULL,
    owner_type                  smallint      NOT NULL,
    -- only non-null for objects with an owner,
    -- the owner can be an account or an object. 
    owner_id                    bytea,
    serialized_object           bytea[]       NOT NULL,
    coin_type                   text,
    coin_balance                bigint,
    dynamic_field_name_type     text,
    dynamic_field_value         text,
    dynamic_field_type          smallint
);

CREATE INDEX objects_owner ON objects (owner_type, owner_id);
CREATE INDEX objects_coin ON objects USING HASH (coin_type);
