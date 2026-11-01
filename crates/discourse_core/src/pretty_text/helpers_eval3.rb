proc do |slug, cooking_user_id, types_in_priority_order|
  cooking_user = User.find_by(id: cooking_user_id)
  types_in_priority_order = types_in_priority_order.select do |type|
    HashtagAutocompleteService.data_source_types.include?(type)
  end
  result = HashtagAutocompleteService.new(Guardian.new(cooking_user)).lookup([slug], types_in_priority_order)
  found_hashtag = nil
  types_in_priority_order.each do |type|
    if result[type.to_sym].any?
      found_hashtag = result[type.to_sym].first.to_h
      break
    end
  end
  found_hashtag
end
