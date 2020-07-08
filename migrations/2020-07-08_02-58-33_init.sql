-- Add migration script here
create table files
(
	id bigserial not null
		constraint files_pk
			primary key,
	hash varchar,
	mime text,
	ext text,
	ip text,
	deleted boolean default false
);

alter table files owner to postgres;

create unique index files_hash_uindex
	on files (hash);

create unique index files_id_uindex
	on files (id);

