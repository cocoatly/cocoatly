CREATE TABLE IF NOT EXISTS packages (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text UNIQUE NOT NULL,
  description text,
  homepage text,
  repository text,
  license text,
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now(),
  downloads_total bigint DEFAULT 0,
  is_deprecated boolean DEFAULT false
);

CREATE TABLE IF NOT EXISTS package_versions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
  version_major integer NOT NULL,
  version_minor integer NOT NULL,
  version_patch integer NOT NULL,
  prerelease text,
  build_metadata text,
  download_url text NOT NULL,
  checksum text NOT NULL,
  checksum_algorithm text NOT NULL,
  signature text,
  size_bytes bigint NOT NULL,
  published_at timestamptz DEFAULT now(),
  yanked boolean DEFAULT false,
  readme text,
  UNIQUE(package_id, version_major, version_minor, version_patch, prerelease)
);

CREATE TABLE IF NOT EXISTS package_authors (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
  author_name text NOT NULL,
  author_email text,
  created_at timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS package_keywords (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
  keyword text NOT NULL,
  created_at timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS package_categories (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
  category text NOT NULL,
  created_at timestamptz DEFAULT now()
);

CREATE TABLE IF NOT EXISTS dependencies (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  version_id uuid NOT NULL REFERENCES package_versions(id) ON DELETE CASCADE,
  dependency_name text NOT NULL,
  version_requirement text NOT NULL,
  dependency_type text NOT NULL,
  optional boolean DEFAULT false,
  features jsonb DEFAULT '[]'::jsonb
);

CREATE TABLE IF NOT EXISTS download_stats (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  package_id uuid NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
  version_id uuid REFERENCES package_versions(id) ON DELETE CASCADE,
  downloaded_at timestamptz DEFAULT now(),
  client_ip inet,
  user_agent text,
  country_code text
);

CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
CREATE INDEX IF NOT EXISTS idx_packages_created_at ON packages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX IF NOT EXISTS idx_package_versions_published_at ON package_versions(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_package_keywords_keyword ON package_keywords(keyword);
CREATE INDEX IF NOT EXISTS idx_package_categories_category ON package_categories(category);
CREATE INDEX IF NOT EXISTS idx_dependencies_version_id ON dependencies(version_id);
CREATE INDEX IF NOT EXISTS idx_dependencies_dependency_name ON dependencies(dependency_name);
CREATE INDEX IF NOT EXISTS idx_download_stats_package_id ON download_stats(package_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_downloaded_at ON download_stats(downloaded_at DESC);

ALTER TABLE packages ENABLE ROW LEVEL SECURITY;
ALTER TABLE package_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE package_authors ENABLE ROW LEVEL SECURITY;
ALTER TABLE package_keywords ENABLE ROW LEVEL SECURITY;
ALTER TABLE package_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE dependencies ENABLE ROW LEVEL SECURITY;
ALTER TABLE download_stats ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Public read access to packages"
  ON packages FOR SELECT
  TO anon, authenticated
  USING (true);

CREATE POLICY "Authenticated users can insert packages"
  ON packages FOR INSERT
  TO authenticated
  WITH CHECK (true);

CREATE POLICY "Authenticated users can update their packages"
  ON packages FOR UPDATE
  TO authenticated
  USING (true)
  WITH CHECK (true);

CREATE POLICY "Public read access to package_versions"
  ON package_versions FOR SELECT
  TO anon, authenticated
  USING (true);

CREATE POLICY "Authenticated users can insert package_versions"
  ON package_versions FOR INSERT
  TO authenticated
  WITH CHECK (true);

CREATE POLICY "Public read access to package_authors"
  ON package_authors FOR SELECT
  TO anon, authenticated
  USING (true);

CREATE POLICY "Authenticated users can insert package_authors"
  ON package_authors FOR INSERT
  TO authenticated
  WITH CHECK (true);

CREATE POLICY "Public read access to package_keywords"
  ON package_keywords FOR SELECT
  TO anon, authenticated
  USING (true);

CREATE POLICY "Authenticated users can insert package_keywords"
  ON package_keywords FOR INSERT
  TO authenticated
  WITH CHECK (true);

CREATE POLICY "Public read access to package_categories"
  ON package_categories FOR SELECT
  TO anon, authenticated
  USING (true);

CREATE POLICY "Authenticated users can insert package_categories"
  ON package_categories FOR INSERT
  TO authenticated
  WITH CHECK (true);

CREATE POLICY "Public read access to dependencies"
  ON dependencies FOR SELECT
  TO anon, authenticated
  USING (true);

CREATE POLICY "Authenticated users can insert dependencies"
  ON dependencies FOR INSERT
  TO authenticated
  WITH CHECK (true);

CREATE POLICY "Public read access to download_stats"
  ON download_stats FOR SELECT
  TO anon, authenticated
  USING (true);

CREATE POLICY "Authenticated users can insert download_stats"
  ON download_stats FOR INSERT
  TO authenticated
  WITH CHECK (true);
