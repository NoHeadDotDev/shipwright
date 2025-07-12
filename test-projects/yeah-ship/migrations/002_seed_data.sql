{%- if database != "none" %}
-- Seed data migration
-- This migration populates the database with initial data for development and testing

{%- if database == "postgres" %}
-- Insert sample users
INSERT INTO users (id, username, email, password_hash, first_name, last_name, is_active, is_verified) VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin', 'admin@{{project-name}}.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Admin', 'User', true, true),
    ('00000000-0000-0000-0000-000000000002', 'johndoe', 'john@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'John', 'Doe', true, true),
    ('00000000-0000-0000-0000-000000000003', 'janedoe', 'jane@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Jane', 'Doe', true, true),
    ('00000000-0000-0000-0000-000000000004', 'testuser', 'test@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Test', 'User', true, false);

{%- elif database == "sqlite" %}
-- Insert sample users for SQLite
INSERT INTO users (id, username, email, password_hash, first_name, last_name, is_active, is_verified) VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin', 'admin@{{project-name}}.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Admin', 'User', 1, 1),
    ('00000000-0000-0000-0000-000000000002', 'johndoe', 'john@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'John', 'Doe', 1, 1),
    ('00000000-0000-0000-0000-000000000003', 'janedoe', 'jane@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Jane', 'Doe', 1, 1),
    ('00000000-0000-0000-0000-000000000004', 'testuser', 'test@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Test', 'User', 1, 0);

{%- elif database == "mysql" %}
-- Insert sample users for MySQL
INSERT INTO users (id, username, email, password_hash, first_name, last_name, is_active, is_verified) VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin', 'admin@{{project-name}}.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Admin', 'User', TRUE, TRUE),
    ('00000000-0000-0000-0000-000000000002', 'johndoe', 'john@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'John', 'Doe', TRUE, TRUE),
    ('00000000-0000-0000-0000-000000000003', 'janedoe', 'jane@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Jane', 'Doe', TRUE, TRUE),
    ('00000000-0000-0000-0000-000000000004', 'testuser', 'test@example.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/lewdoqJ/OOQ3V8K0y', 'Test', 'User', TRUE, FALSE);
{%- endif %}

-- Insert sample tags
{%- if database == "postgres" %}
INSERT INTO tags (id, name, slug, description) VALUES
    ('10000000-0000-0000-0000-000000000001', 'Technology', 'technology', 'Posts about technology and programming'),
    ('10000000-0000-0000-0000-000000000002', 'Web Development', 'web-development', 'Posts about web development techniques'),
    ('10000000-0000-0000-0000-000000000003', 'Rust', 'rust', 'Posts about the Rust programming language'),
    ('10000000-0000-0000-0000-000000000004', 'LiveView', 'liveview', 'Posts about LiveView and real-time web applications'),
    ('10000000-0000-0000-0000-000000000005', 'Tutorial', 'tutorial', 'Step-by-step tutorials and guides'),
    ('10000000-0000-0000-0000-000000000006', 'Best Practices', 'best-practices', 'Best practices and coding standards');

{%- elif database == "sqlite" %}
INSERT INTO tags (id, name, slug, description) VALUES
    ('10000000-0000-0000-0000-000000000001', 'Technology', 'technology', 'Posts about technology and programming'),
    ('10000000-0000-0000-0000-000000000002', 'Web Development', 'web-development', 'Posts about web development techniques'),
    ('10000000-0000-0000-0000-000000000003', 'Rust', 'rust', 'Posts about the Rust programming language'),
    ('10000000-0000-0000-0000-000000000004', 'LiveView', 'liveview', 'Posts about LiveView and real-time web applications'),
    ('10000000-0000-0000-0000-000000000005', 'Tutorial', 'tutorial', 'Step-by-step tutorials and guides'),
    ('10000000-0000-0000-0000-000000000006', 'Best Practices', 'best-practices', 'Best practices and coding standards');

{%- elif database == "mysql" %}
INSERT INTO tags (id, name, slug, description) VALUES
    ('10000000-0000-0000-0000-000000000001', 'Technology', 'technology', 'Posts about technology and programming'),
    ('10000000-0000-0000-0000-000000000002', 'Web Development', 'web-development', 'Posts about web development techniques'),
    ('10000000-0000-0000-0000-000000000003', 'Rust', 'rust', 'Posts about the Rust programming language'),
    ('10000000-0000-0000-0000-000000000004', 'LiveView', 'liveview', 'Posts about LiveView and real-time web applications'),
    ('10000000-0000-0000-0000-000000000005', 'Tutorial', 'tutorial', 'Step-by-step tutorials and guides'),
    ('10000000-0000-0000-0000-000000000006', 'Best Practices', 'best-practices', 'Best practices and coding standards');
{%- endif %}

-- Insert sample posts
{%- if database == "postgres" %}
INSERT INTO posts (id, author_id, title, slug, content, excerpt, status, published_at) VALUES
    ('20000000-0000-0000-0000-000000000001', 
     '00000000-0000-0000-0000-000000000001', 
     'Welcome to {{project-name}}', 
     'welcome-to-{{project-name}}',
     'This is the first post in your new {{project-name}} application. It demonstrates the basic functionality of the content management system built with Rust and LiveView.

## Features

- **Real-time Updates**: LiveView provides instant updates without page refreshes
- **Type Safety**: Rust''s type system prevents runtime errors
- **Performance**: Built for speed with async Rust
- **Scalability**: Designed to handle high traffic loads

## Getting Started

To create new posts, use the admin interface or the REST API endpoints. All content is stored securely in the database with proper validation and sanitization.',
     'Welcome to your new {{project-name}} application! This post introduces the key features and capabilities.',
     'published',
     CURRENT_TIMESTAMP - INTERVAL '1 day'),
    
    ('20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000002',
     'Building Real-time Web Applications with LiveView',
     'building-realtime-web-applications-with-liveview',
     'LiveView is a revolutionary approach to building interactive web applications. Instead of writing complex JavaScript, you can build rich, real-time user interfaces using only server-side code.

## Key Benefits

1. **Simplified Development**: No need to manage client-server state synchronization
2. **Real-time by Default**: WebSocket connections provide instant updates
3. **SEO Friendly**: Server-side rendering ensures search engines can index your content
4. **Reduced Complexity**: Single language and runtime for both frontend and backend

## Example Implementation

```rust
#[derive(LiveView)]
struct Counter {
    count: i32,
}

impl LiveView for Counter {
    type Message = CounterMessage;
    
    fn update(mut self, msg: Self::Message, _data: Option<EventData>) -> Updated<Self> {
        match msg {
            CounterMessage::Increment => self.count += 1,
            CounterMessage::Decrement => self.count -= 1,
        }
        Updated::new(self)
    }
    
    fn render(&self) -> Html<Self::Message> {
        html! {
            <div>
                <h1>{ self.count }</h1>
                <button axm-click={ CounterMessage::Increment }>"+"</button>
                <button axm-click={ CounterMessage::Decrement }>"-"</button>
            </div>
        }
    }
}
```

This simple example demonstrates how easy it is to create interactive components with LiveView.',
     'Learn how to build real-time web applications using LiveView and server-side rendering.',
     'published',
     CURRENT_TIMESTAMP - INTERVAL '2 hours'),
     
    ('20000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000003',
     'Rust Web Development Best Practices',
     'rust-web-development-best-practices',
     'Developing web applications in Rust requires understanding both the language''s unique features and web development principles. Here are some best practices to follow:

## Project Structure

Organize your Rust web application into logical modules:

```
src/
├── main.rs
├── controllers/
│   ├── mod.rs
│   ├── users.rs
│   └── posts.rs
├── models/
│   ├── mod.rs
│   ├── user.rs
│   └── post.rs
├── services/
│   ├── mod.rs
│   ├── auth.rs
│   └── email.rs
└── utils/
    ├── mod.rs
    └── validation.rs
```

## Error Handling

Use custom error types with thiserror:

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found")]
    NotFound,
}
```

## Testing

Write comprehensive tests for your application:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_user() {
        let pool = setup_test_db().await;
        let user = create_user(&pool, "testuser", "test@example.com").await.unwrap();
        assert_eq!(user.username, "testuser");
    }
}
```

## Performance

- Use connection pooling for database access
- Implement proper caching strategies
- Profile your application regularly
- Use async/await for I/O operations

Following these practices will help you build maintainable and scalable Rust web applications.',
     'Essential best practices for building robust and maintainable Rust web applications.',
     'published',
     CURRENT_TIMESTAMP - INTERVAL '1 week'),
     
    ('20000000-0000-0000-0000-000000000004',
     '00000000-0000-0000-0000-000000000001',
     'Database Migration Strategies',
     'database-migration-strategies',
     'Managing database schema changes is crucial for any web application. This post covers strategies for handling database migrations effectively.

## Migration Best Practices

1. **Version Control**: Keep all migrations in version control
2. **Backward Compatibility**: Ensure migrations can be rolled back safely
3. **Testing**: Test migrations on a copy of production data
4. **Atomic Changes**: Make each migration atomic and focused

## Example Migration

```sql
-- 003_add_user_preferences.sql
CREATE TABLE user_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    preference_key VARCHAR(100) NOT NULL,
    preference_value TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, preference_key)
);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
```

This migration adds a new table for storing user preferences with proper foreign key constraints and indexes.',
     'Learn effective strategies for managing database schema changes and migrations.',
     'draft',
     NULL);

{%- elif database == "sqlite" %}
INSERT INTO posts (id, author_id, title, slug, content, excerpt, status, published_at) VALUES
    ('20000000-0000-0000-0000-000000000001', 
     '00000000-0000-0000-0000-000000000001', 
     'Welcome to {{project-name}}', 
     'welcome-to-{{project-name}}',
     'This is the first post in your new {{project-name}} application...',
     'Welcome to your new {{project-name}} application!',
     'published',
     datetime('now', '-1 day')),
    
    ('20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000002',
     'Building Real-time Web Applications with LiveView',
     'building-realtime-web-applications-with-liveview',
     'LiveView is a revolutionary approach to building interactive web applications...',
     'Learn how to build real-time web applications using LiveView.',
     'published',
     datetime('now', '-2 hours')),
     
    ('20000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000003',
     'Rust Web Development Best Practices',
     'rust-web-development-best-practices',
     'Developing web applications in Rust requires understanding...',
     'Essential best practices for building robust Rust web applications.',
     'published',
     datetime('now', '-1 week')),
     
    ('20000000-0000-0000-0000-000000000004',
     '00000000-0000-0000-0000-000000000001',
     'Database Migration Strategies',
     'database-migration-strategies',
     'Managing database schema changes is crucial...',
     'Learn effective strategies for managing database migrations.',
     'draft',
     NULL);

{%- elif database == "mysql" %}
INSERT INTO posts (id, author_id, title, slug, content, excerpt, status, published_at) VALUES
    ('20000000-0000-0000-0000-000000000001', 
     '00000000-0000-0000-0000-000000000001', 
     'Welcome to {{project-name}}', 
     'welcome-to-{{project-name}}',
     'This is the first post in your new {{project-name}} application...',
     'Welcome to your new {{project-name}} application!',
     'published',
     DATE_SUB(NOW(), INTERVAL 1 DAY)),
    
    ('20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000002',
     'Building Real-time Web Applications with LiveView',
     'building-realtime-web-applications-with-liveview',
     'LiveView is a revolutionary approach to building interactive web applications...',
     'Learn how to build real-time web applications using LiveView.',
     'published',
     DATE_SUB(NOW(), INTERVAL 2 HOUR)),
     
    ('20000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000003',
     'Rust Web Development Best Practices',
     'rust-web-development-best-practices',
     'Developing web applications in Rust requires understanding...',
     'Essential best practices for building robust Rust web applications.',
     'published',
     DATE_SUB(NOW(), INTERVAL 1 WEEK)),
     
    ('20000000-0000-0000-0000-000000000004',
     '00000000-0000-0000-0000-000000000001',
     'Database Migration Strategies',
     'database-migration-strategies',
     'Managing database schema changes is crucial...',
     'Learn effective strategies for managing database migrations.',
     'draft',
     NULL);
{%- endif %}

-- Link posts to tags
INSERT INTO post_tags (post_id, tag_id) VALUES
    ('20000000-0000-0000-0000-000000000001', '10000000-0000-0000-0000-000000000001'), -- Welcome -> Technology
    ('20000000-0000-0000-0000-000000000002', '10000000-0000-0000-0000-000000000002'), -- LiveView -> Web Development
    ('20000000-0000-0000-0000-000000000002', '10000000-0000-0000-0000-000000000004'), -- LiveView -> LiveView
    ('20000000-0000-0000-0000-000000000002', '10000000-0000-0000-0000-000000000005'), -- LiveView -> Tutorial
    ('20000000-0000-0000-0000-000000000003', '10000000-0000-0000-0000-000000000003'), -- Best Practices -> Rust
    ('20000000-0000-0000-0000-000000000003', '10000000-0000-0000-0000-000000000006'), -- Best Practices -> Best Practices
    ('20000000-0000-0000-0000-000000000004', '10000000-0000-0000-0000-000000000001'), -- Migration -> Technology
    ('20000000-0000-0000-0000-000000000004', '10000000-0000-0000-0000-000000000006'); -- Migration -> Best Practices

-- Insert sample comments
{%- if database == "postgres" %}
INSERT INTO comments (id, post_id, author_id, content, is_approved) VALUES
    ('30000000-0000-0000-0000-000000000001',
     '20000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000002',
     'Great introduction to the platform! I''m excited to see what we can build with this.',
     true),
     
    ('30000000-0000-0000-0000-000000000002',
     '20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000003',
     'LiveView is indeed a game-changer. The real-time capabilities are impressive!',
     true),
     
    ('30000000-0000-0000-0000-000000000003',
     '20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000001',
     'Thanks for the detailed explanation. The code examples are very helpful.',
     true),
     
    ('30000000-0000-0000-0000-000000000004',
     '20000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000004',
     'These best practices will definitely help new Rust developers. Well written!',
     true);

{%- elif database == "sqlite" %}
INSERT INTO comments (id, post_id, author_id, content, is_approved) VALUES
    ('30000000-0000-0000-0000-000000000001',
     '20000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000002',
     'Great introduction to the platform!',
     1),
     
    ('30000000-0000-0000-0000-000000000002',
     '20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000003',
     'LiveView is indeed a game-changer!',
     1),
     
    ('30000000-0000-0000-0000-000000000003',
     '20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000001',
     'Thanks for the detailed explanation.',
     1),
     
    ('30000000-0000-0000-0000-000000000004',
     '20000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000004',
     'These best practices are very helpful!',
     1);

{%- elif database == "mysql" %}
INSERT INTO comments (id, post_id, author_id, content, is_approved) VALUES
    ('30000000-0000-0000-0000-000000000001',
     '20000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000002',
     'Great introduction to the platform!',
     TRUE),
     
    ('30000000-0000-0000-0000-000000000002',
     '20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000003',
     'LiveView is indeed a game-changer!',
     TRUE),
     
    ('30000000-0000-0000-0000-000000000003',
     '20000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000001',
     'Thanks for the detailed explanation.',
     TRUE),
     
    ('30000000-0000-0000-0000-000000000004',
     '20000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000004',
     'These best practices are very helpful!',
     TRUE);
{%- endif %}

-- Insert sample activity logs
{%- if database == "postgres" %}
INSERT INTO activity_logs (id, user_id, action, entity_type, entity_id, metadata) VALUES
    ('40000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000001',
     'user_login',
     'user',
     '00000000-0000-0000-0000-000000000001',
     '{"login_method": "password", "success": true}'::jsonb),
     
    ('40000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000001',
     'post_created',
     'post',
     '20000000-0000-0000-0000-000000000001',
     '{"title": "Welcome to {{project-name}}", "status": "published"}'::jsonb),
     
    ('40000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000002',
     'comment_created',
     'comment',
     '30000000-0000-0000-0000-000000000001',
     '{"post_id": "20000000-0000-0000-0000-000000000001", "approved": true}'::jsonb);

{%- elif database == "sqlite" %}
INSERT INTO activity_logs (id, user_id, action, entity_type, entity_id, metadata) VALUES
    ('40000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000001',
     'user_login',
     'user',
     '00000000-0000-0000-0000-000000000001',
     '{"login_method": "password", "success": true}'),
     
    ('40000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000001',
     'post_created',
     'post',
     '20000000-0000-0000-0000-000000000001',
     '{"title": "Welcome to {{project-name}}", "status": "published"}'),
     
    ('40000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000002',
     'comment_created',
     'comment',
     '30000000-0000-0000-0000-000000000001',
     '{"post_id": "20000000-0000-0000-0000-000000000001", "approved": true}');

{%- elif database == "mysql" %}
INSERT INTO activity_logs (id, user_id, action, entity_type, entity_id, metadata) VALUES
    ('40000000-0000-0000-0000-000000000001',
     '00000000-0000-0000-0000-000000000001',
     'user_login',
     'user',
     '00000000-0000-0000-0000-000000000001',
     JSON_OBJECT('login_method', 'password', 'success', true)),
     
    ('40000000-0000-0000-0000-000000000002',
     '00000000-0000-0000-0000-000000000001',
     'post_created',
     'post',
     '20000000-0000-0000-0000-000000000001',
     JSON_OBJECT('title', 'Welcome to {{project-name}}', 'status', 'published')),
     
    ('40000000-0000-0000-0000-000000000003',
     '00000000-0000-0000-0000-000000000002',
     'comment_created',
     'comment',
     '30000000-0000-0000-0000-000000000001',
     JSON_OBJECT('post_id', '20000000-0000-0000-0000-000000000001', 'approved', true));
{%- endif %}
{%- endif %}