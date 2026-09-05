use std::borrow::Cow;

#[cfg(feature = "diesel")]
use diesel::{AsChangeset, Insertable};
#[cfg(feature = "diesel")]
use diesel_migrations::{EmbeddedMigrations, embed_migrations};
use lockinspiel_common_schema::error::EyreErrorWrapper;
use serde::{Deserialize, Serialize};
use utoipa::IntoResponses;
#[cfg(feature = "utoipa")]
use utoipa::ToSchema;
use utoipa_e2e::IntoPath;
use uuid::Uuid;

#[cfg(feature = "diesel")]
pub mod schema;

#[cfg(feature = "diesel")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[cfg_attr(feature = "diesel", derive(Insertable))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(
    feature = "utoipa",
    schema(examples(InsertableUserProfile::placeholder))
)]
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[cfg_attr(feature = "diesel", diesel(table_name = schema::user::profiles))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct InsertableUserProfile<'a> {
    pub display_name: Cow<'a, str>,
    pub bio: Cow<'a, str>,
}

impl InsertableUserProfile<'_> {
    pub fn placeholder() -> Self {
        Self {
            display_name: Cow::Borrowed("John Doe"),
            bio: Cow::Borrowed("I wonder how Alice and Bob are doing"),
        }
    }
}

#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Deserialize, Serialize, Debug)]
#[cfg_attr(feature = "utoipa", schema(examples(UserProfile::placeholder)))]
pub struct UserProfile<'a> {
    pub user_id: Uuid,
    pub display_name: Cow<'a, str>,
    pub bio: Cow<'a, str>,
    pub avatar_location: Option<Cow<'a, str>>,
}

impl UserProfile<'_> {
    pub fn placeholder() -> Self {
        Self {
            user_id: Uuid::nil(),
            display_name: Cow::Borrowed("John Doe"),
            bio: Cow::Borrowed("I wonder how Alice and Bob are doing"),
            avatar_location: Some(Cow::Borrowed("/user/profile")),
        }
    }
}

#[cfg_attr(feature = "diesel", derive(AsChangeset))]
#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[cfg_attr(
    feature = "utoipa",
    schema(examples(UserProfileChangeset::placeholder))
)]
#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[cfg_attr(feature = "diesel", diesel(table_name = schema::user::profiles))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct UserProfileChangeset<'a> {
    pub display_name: Cow<'a, str>,
    pub bio: Cow<'a, str>,
}

impl UserProfileChangeset<'_> {
    pub fn placeholder() -> Self {
        Self {
            display_name: Cow::Borrowed("John Doe"),
            bio: Cow::Borrowed("I wonder how Alice and Bob are doing"),
        }
    }
}

#[cfg_attr(feature = "utoipa", derive(ToSchema))]
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct UserAvatarPutUrl<'a> {
    pub url: Cow<'a, str>
}

#[derive(Deserialize, Serialize, ToSchema, Debug)]
#[cfg_attr(
    feature = "utoipa",
    schema(examples(PutAvatarQuery::placeholder))
)]
pub struct PutAvatarQuery<'a> {
    pub file_extension: Cow<'a, str>,
}

impl PutAvatarQuery<'_> {
    pub fn placeholder() -> Self {
        Self {
            file_extension: Cow::Borrowed("png"),
        }
    }
}


#[derive(IntoResponses, ToSchema)]
pub enum UserSchemaResponsesVoid<'a> {
    #[response(status = 200)]
    Success(()),
    #[response(
        status = "4XX",
        description = "It's your fault"
    )]
    YourFault(EyreErrorWrapper<'a>),
    #[response(
        status = "5XX",
        description = "We're having a skill issue"
    )]
    OurFault(EyreErrorWrapper<'a>),
}

#[derive(IntoResponses, ToSchema)]
pub enum UserSchemaResponsesUrl<'a> {
    #[response(status = 200, content_type = "text/plain")]
    Success(UserAvatarPutUrl<'a>),
    #[response(
        status = "4XX",
        description = "It's your fault"
    )]
    YourFault(EyreErrorWrapper<'a>),
    #[response(
        status = "5XX",
        description = "We're having a skill issue"
    )]
    OurFault(EyreErrorWrapper<'a>),
}

#[derive(IntoResponses, ToSchema)]
pub enum UserSchemaUserProfileResponse<'a> {
    #[response(status = 200)]
    Success(UserProfile<'a>),
    #[response(
        status = "4XX",
        description = "It's your fault"
    )]
    YourFault(EyreErrorWrapper<'a>),
    #[response(
        status = "5XX",
        description = "We're having a skill issue"
    )]
    OurFault(EyreErrorWrapper<'a>),
}

#[derive(IntoPath)]
#[api_path(
    post,
    path = "/user/profile",
    tag = "Profile",
    summary = "Create profile",
    description = "Creates a new user profile for the current session",
    responses(UserSchemaResponsesVoid),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct CreateProfileRoute<'a> {
    #[body]
    pub user_profile: InsertableUserProfile<'a>,
}

#[derive(IntoPath)]
#[api_path(
    get,
    path = "/user/profile",
    tag = "Profile",
    summary = "Get profile",
    description = "Gets the user profile for the current session",
    responses(UserSchemaUserProfileResponse),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct GetProfileRoute {}

#[derive(IntoPath)]
#[api_path(
    put,
    path = "/user/profile",
    tag = "Profile",
    summary = "Update profile",
    description = "Updates the user profile for the current session",
    responses(UserSchemaResponsesVoid),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct UpdateProfileRoute<'a> {
    #[body]
    pub changeset: UserProfileChangeset<'a>
}

#[derive(IntoPath)]
#[api_path(   
    put,
    path = "/user/profile/avatar",
    tag = "Profile",
    summary = "Replace profile avatar",
    description = "Returns the URL that should be used to upload an image of the user's new avatar",
    responses(UserSchemaResponsesUrl),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct PutAvatarRoute<'a> {
    #[body]
    pub avatar_query: PutAvatarQuery<'a>,
}

#[derive(IntoPath)]
#[api_path(   
    delete,
    path = "/user/profile/avatar",
    tag = "Profile",
    summary = "Delete profile avatar",
    description = "Delete's the user profile's avatar from wherever it is stored",
    responses(UserSchemaResponsesVoid),
    security(
        ("bearer_jwt" = []),
    )
)]
pub struct DeleteAvatarRoute {}
