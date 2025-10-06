import os
from typing import List, Dict, Any, Optional
from datetime import datetime
from ..core.models import Package, PackageVersion, Version
from ..core.exceptions import RegistryException, PackageNotFoundException

class RegistryAPI:
    def __init__(self):
        supabase_url = os.getenv("VITE_SUPABASE_URL")
        supabase_key = os.getenv("VITE_SUPABASE_ANON_KEY")

        if not supabase_url or not supabase_key:
            raise RegistryException("Supabase credentials not configured")

        from supabase import create_client
        self.supabase = create_client(supabase_url, supabase_key)

    def search_packages(
        self,
        query: str,
        limit: int = 50,
        offset: int = 0,
        categories: Optional[List[str]] = None,
        keywords: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        query_builder = self.supabase.table("packages").select("*")

        if query:
            query_builder = query_builder.ilike("name", f"%{query}%")

        if categories:
            category_filter = self.supabase.table("package_categories").select("package_id").in_("category", categories)
            cat_results = category_filter.execute()
            package_ids = [row["package_id"] for row in cat_results.data]
            if package_ids:
                query_builder = query_builder.in_("id", package_ids)

        result = query_builder.range(offset, offset + limit - 1).execute()

        return {
            "packages": result.data,
            "total": len(result.data),
            "has_more": len(result.data) == limit,
        }

    def get_package(self, package_name: str) -> Dict[str, Any]:
        result = self.supabase.table("packages").select("*").eq("name", package_name).maybeSingle().execute()

        if not result.data:
            raise PackageNotFoundException(package_name)

        package_data = result.data

        versions = self.supabase.table("package_versions").select("*").eq("package_id", package_data["id"]).execute()
        authors = self.supabase.table("package_authors").select("*").eq("package_id", package_data["id"]).execute()
        keywords = self.supabase.table("package_keywords").select("*").eq("package_id", package_data["id"]).execute()
        categories = self.supabase.table("package_categories").select("*").eq("package_id", package_data["id"]).execute()

        package_data["versions"] = versions.data
        package_data["authors"] = [a["author_name"] for a in authors.data]
        package_data["keywords"] = [k["keyword"] for k in keywords.data]
        package_data["categories"] = [c["category"] for c in categories.data]

        return package_data

    def get_package_versions(self, package_name: str) -> List[Dict[str, Any]]:
        package_result = self.supabase.table("packages").select("id").eq("name", package_name).maybeSingle().execute()

        if not package_result.data:
            raise PackageNotFoundException(package_name)

        package_id = package_result.data["id"]

        versions_result = self.supabase.table("package_versions").select("*").eq("package_id", package_id).order("published_at", desc=True).execute()

        return versions_result.data

    def get_package_version(self, package_name: str, version_str: str) -> Dict[str, Any]:
        package_result = self.supabase.table("packages").select("id").eq("name", package_name).maybeSingle().execute()

        if not package_result.data:
            raise PackageNotFoundException(package_name)

        package_id = package_result.data["id"]

        version = Version.parse(version_str)

        version_result = (
            self.supabase.table("package_versions")
            .select("*")
            .eq("package_id", package_id)
            .eq("version_major", version.major)
            .eq("version_minor", version.minor)
            .eq("version_patch", version.patch)
            .maybeSingle()
            .execute()
        )

        if not version_result.data:
            raise PackageNotFoundException(f"{package_name}@{version_str}")

        return version_result.data

    def publish_package(
        self,
        package_data: Dict[str, Any],
        download_url: str,
        checksum: str,
        checksum_algorithm: str,
        size_bytes: int,
    ) -> Dict[str, Any]:
        existing = self.supabase.table("packages").select("id").eq("name", package_data["name"]).maybeSingle().execute()

        if existing.data:
            package_id = existing.data["id"]

            self.supabase.table("packages").update({
                "description": package_data.get("description"),
                "homepage": package_data.get("homepage"),
                "repository": package_data.get("repository"),
                "license": package_data.get("license"),
                "updated_at": datetime.utcnow().isoformat(),
            }).eq("id", package_id).execute()
        else:
            package_result = self.supabase.table("packages").insert({
                "name": package_data["name"],
                "description": package_data.get("description"),
                "homepage": package_data.get("homepage"),
                "repository": package_data.get("repository"),
                "license": package_data.get("license"),
            }).execute()

            package_id = package_result.data[0]["id"]

            if "authors" in package_data:
                for author in package_data["authors"]:
                    self.supabase.table("package_authors").insert({
                        "package_id": package_id,
                        "author_name": author,
                    }).execute()

            if "keywords" in package_data:
                for keyword in package_data["keywords"]:
                    self.supabase.table("package_keywords").insert({
                        "package_id": package_id,
                        "keyword": keyword,
                    }).execute()

            if "categories" in package_data:
                for category in package_data["categories"]:
                    self.supabase.table("package_categories").insert({
                        "package_id": package_id,
                        "category": category,
                    }).execute()

        version = Version.parse(package_data["version"])

        version_result = self.supabase.table("package_versions").insert({
            "package_id": package_id,
            "version_major": version.major,
            "version_minor": version.minor,
            "version_patch": version.patch,
            "prerelease": version.prerelease,
            "build_metadata": version.build,
            "download_url": download_url,
            "checksum": checksum,
            "checksum_algorithm": checksum_algorithm,
            "size_bytes": size_bytes,
            "readme": package_data.get("readme"),
        }).execute()

        version_id = version_result.data[0]["id"]

        if "dependencies" in package_data:
            for dep in package_data["dependencies"]:
                self.supabase.table("dependencies").insert({
                    "version_id": version_id,
                    "dependency_name": dep["name"],
                    "version_requirement": dep["version_requirement"],
                    "dependency_type": dep.get("type", "runtime"),
                    "optional": dep.get("optional", False),
                }).execute()

        return {
            "package_id": package_id,
            "version_id": version_id,
            "message": "Package published successfully",
        }

    def record_download(self, package_name: str, version_str: str, client_info: Dict[str, Any]) -> None:
        package_result = self.supabase.table("packages").select("id").eq("name", package_name).maybeSingle().execute()

        if not package_result.data:
            return

        package_id = package_result.data["id"]

        version = Version.parse(version_str)
        version_result = (
            self.supabase.table("package_versions")
            .select("id")
            .eq("package_id", package_id)
            .eq("version_major", version.major)
            .eq("version_minor", version.minor)
            .eq("version_patch", version.patch)
            .maybeSingle()
            .execute()
        )

        if not version_result.data:
            return

        version_id = version_result.data["id"]

        self.supabase.table("download_stats").insert({
            "package_id": package_id,
            "version_id": version_id,
            "user_agent": client_info.get("user_agent"),
            "country_code": client_info.get("country_code"),
        }).execute()

        self.supabase.rpc("increment_downloads", {"package_id": package_id}).execute()
