-- Migration: 012_force_fix_password_hashes.sql
-- Force update ALL password hashes to valid Argon2id format
-- This replaces ANY existing hash value

-- Update poshan@poshan.com with password: poshan
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDFwb3NoYW4$0MejJJJmDcJflMGshmQ1VK7uhai7dsl6iv3Lzqz30CQ',
    updated_at = NOW()
WHERE email = 'poshan@poshan.com';

-- Update byapak@byapak.com with password: byapak
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDJieWFwYWs$uaR7DUK3P8L5E7dGi6YPAS09v4AoS202U7SJHhPf9MA',
    updated_at = NOW()
WHERE email = 'byapak@byapak.com';

-- Update sandip@sandip.com with password: sandip
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDNzYW5kaXA$oNbwywPxAMCw90lXT9UZMw4fVlGhq+iPeMaWHyvRXBQ',
    updated_at = NOW()
WHERE email = 'sandip@sandip.com';

-- Update testuser1@derivative.com with password: testuser123
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDR0ZXN0MQ$vb0n/AHwgbTG/BBCtcwk0d1PX7KcUzulMvTjcIu0Bjc',
    updated_at = NOW()
WHERE email = 'testuser1@derivative.com';

-- Update testuser2@codeatderivative.com with password: testuser123
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDV0ZXN0Mg$EqyTDYxhJ++J8hXSaAQmE8r/FWUQ0ug9pmMEWkjrVRw',
    updated_at = NOW()
WHERE email = 'testuser2@codeatderivative.com';

-- Show results
SELECT email, 
       CASE WHEN password_hash LIKE '$argon2id$v=19$m=65536,t=3,p=4$c2FsdD%' 
            THEN 'VALID' ELSE 'INVALID' END as hash_status
FROM user_profiles;
