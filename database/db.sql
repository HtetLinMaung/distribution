CREATE TABLE brands
(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);


CREATE TABLE categories
(
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    brand_id INTEGER REFERENCES brands(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);


CREATE TABLE roles
(
    id SERIAL PRIMARY KEY,
    role_name VARCHAR(50) NOT NULL UNIQUE,
    deleted_at TIMESTAMP DEFAULT null
);

INSERT INTO roles
    (role_name)
VALUES
    ('Admin'),
    ('Manager'),
    ('Distributor');

CREATE TABLE users
(
    id SERIAL PRIMARY KEY,
    name varchar(255) not null,
    username VARCHAR(100) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    role_id INTEGER REFERENCES roles(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

insert into users
    (username, password, name, role_id, created_at)
values
    ('admin', '$2b$12$VsrfBeuszFplm3HX4QgMWOg/KMsIhZgPLCjej2W3DI.YHz9Gq9Zjq', 'Admin', 1, now());


CREATE TABLE products
(
    id SERIAL PRIMARY KEY,
    name varchar(255) not null,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    quantity INTEGER NOT NULL,
    brand_id INTEGER REFERENCES brands(id),
    category_id INTEGER REFERENCES categories(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE shops
(
    id SERIAL PRIMARY KEY,
    day varchar(50) not null,
    ward_id INTEGER REFERENCES wards(id),
    name varchar(255) not null,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE wards
(
    id SERIAL PRIMARY KEY,
    name varchar(255) not null,
    township_id INTEGER REFERENCES townships(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);

CREATE TABLE townships
(
    id SERIAL PRIMARY KEY,
    name varchar(50) not null,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT null
);