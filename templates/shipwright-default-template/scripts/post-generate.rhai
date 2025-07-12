// Post-generation script for Shipwright template
let project_name = variable::get("project-name");
let project_type = variable::get("project_type");
let database = variable::get("database");

// Initialize git repository
system::command("git", ["init"]);


// Git operations
system::command("git", ["add", "."]);

// Final message
print("\n✅ Project '" + project_name + "' created successfully!");
print("\n📝 Next steps:");
print("   1. cd " + project_name);
if database != "none" {
    print("   2. shipwright dev");
} else {
    print("   2. shipwright dev");
}
print("\n🚀 Happy coding!");