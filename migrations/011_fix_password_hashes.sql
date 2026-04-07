-- Migration: 011_fix_password_hashes.sql
-- Fix: Update invalid password hashes to proper Argon2id format
-- Note: These are the same passwords as the usernames for development/testing

-- Update poshan@poshan.com with password: poshan
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDFwb3NoYW4$0MejJJJmDcJflMGshmQ1VK7uhai7dsl6iv3Lzqz30CQ',
    updated_at = NOW()
WHERE email = 'poshan@poshan.com'
  AND password_hash LIKE '%IMPORTED_USER_NEEDS_PASSWORD_RESET%';

-- Update byapak@byapak.com with password: byapak
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDJieWFwYWs$uaR7DUK3P8L5E7dGi6YPAS09v4AoS202U7SJHhPf9MA',
    updated_at = NOW()
WHERE email = 'byapak@byapak.com'
  AND password_hash LIKE '%IMPORTED_USER_NEEDS_PASSWORD_RESET%';

-- Update sandip@sandip.com with password: sandip
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDNzYW5kaXA$oNbwywPxAMCw90lXT9UZMw4fVlGhq+iPeMaWHyvRXBQ',
    updated_at = NOW()
WHERE email = 'sandip@sandip.com'
  AND password_hash LIKE '%IMPORTED_USER_NEEDS_PASSWORD_RESET%';

-- Update testuser1@derivative.com with password: testuser123
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDR0ZXN0MQ$vb0n/AHwgbTG/BBCtcwk0d1PX7KcUzulMvTjcIu0Bjc',
    updated_at = NOW()
WHERE email = 'testuser1@derivative.com'
  AND password_hash LIKE '%IMPORTED_USER_NEEDS_PASSWORD_RESET%';

-- Update testuser2@codeatderivative.com with password: testuser123
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDV0ZXN0Mg$EqyTDYxhJ++J8hXSaAQmE8r/FWUQ0ug9pmMEWkjrVRw',
    updated_at = NOW()
WHERE email = 'testuser2@codeatderivative.com'
  AND password_hash LIKE '%IMPORTED_USER_NEEDS_PASSWORD_RESET%';

-- IMPORTANT: For production, users should reset their passwords!
-- These default passwords are only for development/testing purposes.
