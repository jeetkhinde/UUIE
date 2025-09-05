-- Users table definition
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    avatar_url VARCHAR(500),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- User card component template
CREATE COMPONENT user_card AS '
<div class="bg-white rounded-lg shadow-md p-6">
    <div class="flex items-center space-x-4">
        {avatar_url}
        <div>
            {name}
            {email}
            {created_at}
        </div>
    </div>
</div>';