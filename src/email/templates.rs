pub fn verification_email(name: Option<&str>, url: &str) -> String {
    let greeting = name.map(|n| format!("Hi {n},")).unwrap_or_else(|| "Hi,".into());
    format!(
        r#"<!DOCTYPE html>
<html><body style="font-family: -apple-system, sans-serif; max-width: 600px; margin: 40px auto; padding: 20px; color: #1a1a1a;">
<h1 style="font-size: 20px;">Welcome to AaaS</h1>
<p>{greeting}</p>
<p>Confirm your email address to activate your account.</p>
<p style="margin: 30px 0;">
  <a href="{url}" style="display: inline-block; background: #4f46e5; color: #fff; padding: 12px 24px; border-radius: 8px; text-decoration: none; font-weight: 600;">Verify email</a>
</p>
<p style="font-size: 12px; color: #6b7280;">This link expires in 24 hours.</p>
<hr style="border: none; border-top: 1px solid #e5e7eb; margin: 30px 0;">
<p style="font-size: 11px; color: #9ca3af;">If you didn't create this account, ignore this email.</p>
</body></html>"#
    )
}

pub fn password_reset_email(name: Option<&str>, url: &str) -> String {
    let greeting = name.map(|n| format!("Hi {n},")).unwrap_or_else(|| "Hi,".into());
    format!(
        r#"<!DOCTYPE html>
<html><body style="font-family: -apple-system, sans-serif; max-width: 600px; margin: 40px auto; padding: 20px; color: #1a1a1a;">
<h1 style="font-size: 20px;">Reset your password</h1>
<p>{greeting}</p>
<p>Click the button below to set a new password. This link is valid for 1 hour.</p>
<p style="margin: 30px 0;">
  <a href="{url}" style="display: inline-block; background: #dc2626; color: #fff; padding: 12px 24px; border-radius: 8px; text-decoration: none; font-weight: 600;">Reset password</a>
</p>
<p style="font-size: 12px; color: #6b7280;">If you didn't request a reset, ignore this email — your password is unchanged.</p>
</body></html>"#
    )
}
