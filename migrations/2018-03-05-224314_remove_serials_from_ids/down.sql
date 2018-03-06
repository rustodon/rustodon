CREATE SEQUENCE accounts_id_seq;
CREATE SEQUENCE follows_id_seq;
CREATE SEQUENCE statuses_id_seq;
CREATE SEQUENCE users_id_seq;

ALTER TABLE accounts ALTER COLUMN id SET DEFAULT nextval('accounts_id_seq'::regclass);
ALTER TABLE follows ALTER COLUMN id SET DEFAULT nextval('follows_id_seq'::regclass);
ALTER TABLE statuses ALTER COLUMN id SET DEFAULT nextval('statuses_id_seq'::regclass);
ALTER TABLE users ALTER COLUMN id SET DEFAULT nextval('users_id_seq'::regclass);
