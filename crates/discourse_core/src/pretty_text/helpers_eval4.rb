proc do |user_id|
  return unless user_id.is_a?(Integer)
  { staff: User.where(id: user_id).where("moderator OR admin").exists? }
end
