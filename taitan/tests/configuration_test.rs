use taitan::server_conf::ApplicationConfig;

#[test]
pub fn test_configuration_default() {
    let config: ApplicationConfig = ApplicationConfig::default();
    assert_eq!(config.http.http_port, 80);
    assert_eq!(config.http.redirect_https, false);
}

#[test]
pub fn test_configuration_empty() {
    let config: ApplicationConfig = ApplicationConfig::from_toml("");
    assert_eq!(config.logs.logs_dir, "logs/");
}