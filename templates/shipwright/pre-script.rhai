// Pre-generation script for Shipwright template
// This script runs before the template is generated

print("🚢 Initializing Shipwright project template...");

// Validate project name
let project_name = variable::get("project-name");
if project_name.is_empty() {
    print("❌ Project name cannot be empty");
    exit(1);
}

// Validate crate name
let crate_name = variable::get("crate-name");
if crate_name.is_empty() {
    variable::set("crate-name", project_name.replace("-", "_"));
}

print(`✅ Project: ${project_name}`);
print(`✅ Crate: ${variable::get("crate-name")}`);

// Check selected features
if variable::get("use-database") {
    print(`✅ Database: ${variable::get("database-type")}`);
}
if variable::get("use-web") {
    print("✅ Web server: enabled");
}
if variable::get("use-config") {
    print("✅ Configuration: enabled");
}