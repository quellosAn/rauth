
CREATE TABLE application_user (
    user_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    access_failed_count INT NOT NULL,
    email VARCHAR(256) NOT NULL,
    email_confirmed BIT NOT NULL,
    lockout_enabled BIT NULL,
    lockout_end TIMESTAMP NULL,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(512) NOT NULL,
    created_on TIMESTAMP NOT NULL,
    last_modified_on TIMESTAMP NOT NULL ,
    phone_number VARCHAR(50) NULL,
    phone_number_confirmed BIT NULL
);

CREATE TABLE application_role (
    role_id serial PRIMARY KEY,
    name nvarchar(50) NOT NULL UNIQUE,
    created_on TIMESTAMP NOT NULL,
    last_modified_on TIMESTAMP NOT NULL 
)

CREATE TABLE user_role (
    user_id uuid NOT NULL,
    role_id INT NOT NULL,
    PRIMARY KEY (user_id, role_id)
) 

CREATE TABLE persisted_grant (
    grant_id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    consumed_time TIMESTAMP NULL,
    create_time TIMESTAMP NOT NULL,
    description nvarchar(200) NULL,
    expiration TIMESTAMP NOT NULL,
    session_id 

)

CREATE INDEX role_name_index on application_role (name);
CREATE INDEX email_index on application_user (email);
CREATE INDEX username_index on application_user (username);