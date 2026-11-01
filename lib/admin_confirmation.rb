# frozen_string_literal: true

class AdminConfirmation
  attr_accessor :token
  attr_reader :performed_by
  attr_reader :target_user

  def initialize(target_user, performed_by)
    @target_user = target_user
    @performed_by = performed_by
  end




end
