use crate::{ApubObject, Crud};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  naive_now,
  schema::person::dsl::*,
  source::person::{Person, PersonForm},
  DbUrl,
};

mod safe_type {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::person::columns::*, source::person::Person};

  type Columns = (
    id,
    name,
    preferred_username,
    avatar,
    banned,
    published,
    updated,
    actor_id,
    bio,
    local,
    banner,
    deleted,
    inbox_url,
    shared_inbox_url,
  );

  impl ToSafe for Person {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
      )
    }
  }
}

mod safe_type_alias_1 {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::person_alias_1::columns::*, source::person::PersonAlias1};

  type Columns = (
    id,
    name,
    preferred_username,
    avatar,
    banned,
    published,
    updated,
    actor_id,
    bio,
    local,
    banner,
    deleted,
    inbox_url,
    shared_inbox_url,
  );

  impl ToSafe for PersonAlias1 {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
      )
    }
  }
}

mod safe_type_alias_2 {
  use crate::ToSafe;
  use lemmy_db_schema::{schema::person_alias_2::columns::*, source::person::PersonAlias2};

  type Columns = (
    id,
    name,
    preferred_username,
    avatar,
    banned,
    published,
    updated,
    actor_id,
    bio,
    local,
    banner,
    deleted,
    inbox_url,
    shared_inbox_url,
  );

  impl ToSafe for PersonAlias2 {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        preferred_username,
        avatar,
        banned,
        published,
        updated,
        actor_id,
        bio,
        local,
        banner,
        deleted,
        inbox_url,
        shared_inbox_url,
      )
    }
  }
}

impl Crud<PersonForm> for Person {
  fn read(conn: &PgConnection, person_id: i32) -> Result<Self, Error> {
    person
      .filter(deleted.eq(false))
      .find(person_id)
      .first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, person_id: i32) -> Result<usize, Error> {
    diesel::delete(person.find(person_id)).execute(conn)
  }
  fn create(conn: &PgConnection, form: &PersonForm) -> Result<Self, Error> {
    insert_into(person).values(form).get_result::<Self>(conn)
  }
  fn update(conn: &PgConnection, person_id: i32, form: &PersonForm) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}

impl ApubObject<PersonForm> for Person {
  fn read_from_apub_id(conn: &PgConnection, object_id: &DbUrl) -> Result<Self, Error> {
    use lemmy_db_schema::schema::person::dsl::*;
    person
      .filter(deleted.eq(false))
      .filter(actor_id.eq(object_id))
      .first::<Self>(conn)
  }

  fn upsert(conn: &PgConnection, person_form: &PersonForm) -> Result<Person, Error> {
    insert_into(person)
      .values(person_form)
      .on_conflict(actor_id)
      .do_update()
      .set(person_form)
      .get_result::<Self>(conn)
  }
}

pub trait Person_ {
  fn ban_person(conn: &PgConnection, person_id: i32, ban: bool) -> Result<Person, Error>;
  fn find_by_name(conn: &PgConnection, name: &str) -> Result<Person, Error>;
  fn mark_as_updated(conn: &PgConnection, person_id: i32) -> Result<Person, Error>;
  fn delete_account(conn: &PgConnection, person_id: i32) -> Result<Person, Error>;
}

impl Person_ for Person {
  fn ban_person(conn: &PgConnection, person_id: i32, ban: bool) -> Result<Self, Error> {
    diesel::update(person.find(person_id))
      .set(banned.eq(ban))
      .get_result::<Self>(conn)
  }

  fn find_by_name(conn: &PgConnection, from_name: &str) -> Result<Person, Error> {
    person
      .filter(deleted.eq(false))
      .filter(local.eq(true))
      .filter(name.ilike(from_name))
      .first::<Person>(conn)
  }

  fn mark_as_updated(conn: &PgConnection, person_id: i32) -> Result<Person, Error> {
    diesel::update(person.find(person_id))
      .set((last_refreshed_at.eq(naive_now()),))
      .get_result::<Self>(conn)
  }

  fn delete_account(conn: &PgConnection, person_id: i32) -> Result<Person, Error> {
    use lemmy_db_schema::schema::local_user;

    // Set the local user info to none
    diesel::update(local_user::table.filter(local_user::person_id.eq(person_id)))
      .set((
        local_user::email.eq::<Option<String>>(None),
        local_user::matrix_user_id.eq::<Option<String>>(None),
      ))
      .execute(conn)?;

    diesel::update(person.find(person_id))
      .set((
        preferred_username.eq::<Option<String>>(None),
        bio.eq::<Option<String>>(None),
        deleted.eq(true),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{establish_unpooled_connection, source::person::*};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "holly".into(),
      preferred_username: None,
      avatar: None,
      banner: None,
      banned: None,
      deleted: None,
      published: None,
      updated: None,
      actor_id: None,
      bio: None,
      local: None,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      inbox_url: None,
      shared_inbox_url: None,
    };

    let inserted_person = Person::create(&conn, &new_person).unwrap();

    let expected_person = Person {
      id: inserted_person.id,
      name: "holly".into(),
      preferred_username: None,
      avatar: None,
      banner: None,
      banned: false,
      deleted: false,
      published: inserted_person.published,
      updated: None,
      actor_id: inserted_person.actor_id.to_owned(),
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: inserted_person.published,
      inbox_url: inserted_person.inbox_url.to_owned(),
      shared_inbox_url: None,
    };

    let read_person = Person::read(&conn, inserted_person.id).unwrap();
    let updated_person = Person::update(&conn, inserted_person.id, &new_person).unwrap();
    let num_deleted = Person::delete(&conn, inserted_person.id).unwrap();

    assert_eq!(expected_person, read_person);
    assert_eq!(expected_person, inserted_person);
    assert_eq!(expected_person, updated_person);
    assert_eq!(1, num_deleted);
  }
}
