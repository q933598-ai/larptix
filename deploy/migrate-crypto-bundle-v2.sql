BEGIN;

UPDATE crypto_devices
SET bundle_json = json_remove(
    json_set(
        bundle_json,
        '$.one_time_keys',
        json_array(
            json_object(
                'key', json_extract(bundle_json, '$.one_time_key'),
                'signature', json_extract(bundle_json, '$.one_time_signature')
            )
        )
    ),
    '$.one_time_key',
    '$.one_time_signature'
)
WHERE json_type(bundle_json, '$.one_time_keys') IS NULL
  AND json_type(bundle_json, '$.one_time_key') = 'text'
  AND json_type(bundle_json, '$.one_time_signature') = 'text';

COMMIT;
