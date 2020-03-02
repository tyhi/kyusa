create table users
(
	id uuid default uuid_generate_v4(),
	username text not null,
	email text not null,
	apikey text not null,
	ipaddr text not null
);

create unique index users_apikey_uindex
	on users (apikey);

create unique index users_email_uindex
	on users (email);

create unique index users_id_uindex
	on users (id);

create unique index users_username_uindex
	on users (username);

alter table users
	add constraint users_pk
		primary key (id);

create table files
(
	id uuid default uuid_generate_v4() not null,
	owner text not null,
	uploaded timestamp default now() not null,
	path text not null,
	deletekey text not null,
	filesize float8 not null,
	downloads bigint default 0 not null
);

create unique index file_uuid_uindex
	on files (id);

alter table files
	add constraint file_pk
		primary key (id);

