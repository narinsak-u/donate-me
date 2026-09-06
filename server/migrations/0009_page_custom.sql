-- Phase 9: ปรับแต่งหน้าโดเนตเป็นอัตลักษณ์ของสตรีมเมอร์
ALTER TABLE settings ADD COLUMN page_theme TEXT NOT NULL DEFAULT 'rose';    -- rose | mint | midnight | retro
ALTER TABLE settings ADD COLUMN cover_url TEXT NOT NULL DEFAULT '';         -- รูปปก https:// (เว้นว่าง = gradient เดิม)
ALTER TABLE settings ADD COLUMN about_text TEXT NOT NULL DEFAULT '';        -- ≤ 400 ตัวอักษร
ALTER TABLE settings ADD COLUMN social_facebook TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_youtube TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_twitch TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_tiktok TEXT NOT NULL DEFAULT '';
ALTER TABLE settings ADD COLUMN social_x TEXT NOT NULL DEFAULT '';          -- https://x.com/...
