use crate::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::CreateOrUpdateType, objects::page::Page, InCommunity},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::helpers::deserialize_one_or_many,
};
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyResult;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdatePage {
  pub(crate) actor: ObjectId<ApubPerson>,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) to: Vec<Url>,
  pub(crate) object: Page,
  #[serde(deserialize_with = "deserialize_one_or_many")]
  pub(crate) cc: Vec<Url>,
  #[serde(rename = "type")]
  pub(crate) kind: CreateOrUpdateType,
  pub(crate) id: Url,
}

impl InCommunity for CreateOrUpdatePage {
  async fn community(&self, context: &Data<LemmyContext>) -> LemmyResult<ApubCommunity> {
    let community = self.object.community(context).await?;
    Ok(community)
  }
}
