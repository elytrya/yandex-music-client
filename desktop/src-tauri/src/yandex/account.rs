use super::*;

impl Yandex {
    pub async fn account_status(&self) -> Result<(ProfileDto, i64), String> {
        let result = self.get_result("/account/status").await?;
        let status: YAccountStatus =
            serde_json::from_value(result).map_err(|e| e.to_string())?;
        let uid = status.account.uid.unwrap_or_default();
        let avatar_url = status
            .account
            .avatar_url
            .or_else(|| {
                status.account.avatar_id.map(|id| {
                    format!("{}://avatars.yandex.net/get-yapic/{id}/islands-200", "https")
                })
            })
            .map(|url| {
                let normalized = if url.starts_with("//") {
                    format!("{}:{url}", "https")
                } else if !url.starts_with("http://") && !url.starts_with("https://") {
                    format!("{}://{url}", "https")
                } else {
                    url
                };
                normalized.replace("%%", "200x200")
            });
        let profile = ProfileDto {
            uid: status.account.uid,
            login: status.account.login.clone(),
            display_name: status.account.display_name.or(status.account.full_name),
            avatar_url,
            has_plus: status.plus.and_then(|p| p.has_plus).unwrap_or(false),
        };
        Ok((profile, uid))
    }
}
