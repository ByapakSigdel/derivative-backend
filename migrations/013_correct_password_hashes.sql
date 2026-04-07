-- Migration: 013_correct_password_hashes.sql
-- Fix: Correct password hashes (previous ones had newline character in password)

-- Update poshan@poshan.com with password: poshan
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDFwb3NoYW4$QhbRViDIgux03cuF3eeffK+a7wJaNXGk3x8e9dWB8c4',
    updated_at = NOW()
WHERE email = 'poshan@poshan.com';

-- Update byapak@byapak.com with password: byapak
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDJieWFwYWs$begmGJdVDTgO22ZYFxtpryil0RsBwteRmnRuTcnlqPE',
    updated_at = NOW()
WHERE email = 'byapak@byapak.com';

-- Update sandip@sandip.com with password: sandip
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDNzYW5kaXA$pap3DUuF0kXyXz/oBHXI4wCpBfO52zvemvsZg/FZ+DM',
    updated_at = NOW()
WHERE email = 'sandip@sandip.com';

-- Update testuser1@derivative.com with password: testuser123
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDR0ZXN0MQ$jZj+/EHWvF/YlhJzl5v9uwG9yWxVyVJsyF/B9uylSy0',
    updated_at = NOW()
WHERE email = 'testuser1@derivative.com';

-- Update testuser2@codeatderivative.com with password: testuser123
UPDATE public.user_profiles 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c2FsdDV0ZXN0Mg$oHHjT2oXemKtSQnRGMzsNwqY6rgPSGaT4GfSnDGAn6U',
    updated_at = NOW()
WHERE email = 'testuser2@codeatderivative.com';
