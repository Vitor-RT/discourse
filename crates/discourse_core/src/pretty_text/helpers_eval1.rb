proc do |urls|
  map = {}
  result = {}
  urls.each do |url|
    sha1 = Upload.sha1_from_short_url(url)
    if url.split(".")[1].nil?
      thumbnail = Upload.where("original_filename LIKE ?", "#{Upload.sanitize_sql_like(sha1)}.%").last if sha1
      thumbnail = Upload.where("original_filename LIKE ?", "#{Upload.sanitize_sql_like(url)}.%").last if thumbnail.nil? && sha1.nil?
      sha1 = thumbnail.sha1 if thumbnail
    end
    map[url] = sha1 if sha1
  end
  if map.length > 0
    reverse_map = {}
    map.each do |key, value|
      reverse_map[value] ||= []
      reverse_map[value] << key
    end
    Upload.where(sha1: map.values).pluck(:sha1, :url, :extension, :secure).each do |row|
      sha1, url, extension, secure = row
      if short_urls = reverse_map[sha1]
        secure_uploads = SiteSetting.secure_uploads? && secure
        short_urls.each do |short_url|
          result[short_url] = {
            url: (secure_uploads ? Upload.secure_uploads_url_from_upload_url(url) : Discourse.store.cdn_url(url)),
            short_path: Upload.short_path(sha1: sha1, extension: extension),
            base62_sha1: Upload.base62_sha1(sha1),
          }
        end
      end
    end
  end
  result
end
