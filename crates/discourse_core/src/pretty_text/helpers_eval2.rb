proc do |topic_id|
  return unless topic_id.is_a?(Integer)
  topic = Topic.find_by(id: topic_id)
  if topic && Guardian.new.can_see?(topic)
    { title: Rack::Utils.escape_html(topic.title), href: topic.url }
  elsif topic
    { title: I18n.t("on_another_topic"), href: topic.slugless_url }
  end
end
