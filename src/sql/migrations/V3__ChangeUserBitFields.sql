ALTER TABLE application_user
ALTER COLUMN email_confirmed TYPE Boolean using email_confirmed::int8::int4::boolean,
ALTER COLUMN lockout_enabled TYPE Boolean using lockout_enabled::int8::int4::boolean,
ALTER COLUMN phone_number_confirmed TYPE Boolean using phone_number_confirmed::int8::int4::boolean; 