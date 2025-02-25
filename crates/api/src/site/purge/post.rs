use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  send_activity::{ActivityChannel, SendActivityData},
  site::PurgePost,
  utils::is_admin,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    local_user::LocalUser,
    mod_log::admin::{AdminPurgePost, AdminPurgePostForm},
    post::Post,
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn purge_post(
  data: Json<PurgePost>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the post to get the community_id
  let post = Post::read(&mut context.pool(), data.post_id).await?;

  // Also check that you're a higher admin
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![post.creator_id],
  )
  .await?;

  // Purge image
  if let Some(url) = &post.url {
    purge_image_from_pictrs(url, &context).await.ok();
  }
  // Purge thumbnail
  if let Some(thumbnail_url) = &post.thumbnail_url {
    purge_image_from_pictrs(thumbnail_url, &context).await.ok();
  }

  Post::delete(&mut context.pool(), data.post_id).await?;

  // Mod tables
  let form = AdminPurgePostForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    community_id: post.community_id,
  };
  AdminPurgePost::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemovePost {
      post,
      moderator: local_user_view.person.clone(),
      reason: data.reason.clone(),
      removed: true,
    },
    &context,
  )?;

  Ok(Json(SuccessResponse::default()))
}
