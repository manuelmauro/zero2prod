begin;
    -- backfill `status` for historical entries
    update subscriptions
        set status = 'confirmed'
        where status is null;
    -- make `status` mandatory
    alter table subscriptions alter column status set not null;
commit;
