# A realistic web-application controller / service object.

HTTP_OK      = 200
HTTP_NOT_FOUND = 404
DEFAULT_PAGE_SIZE = 25

module Api
  module V1
    class UsersController
      ALLOWED_PARAMS = %w[name email role].freeze

      attr_accessor :current_user, :logger

      def initialize(repo, logger)
        @repo   = repo
        @logger = logger
      end

      def index(params = {})
        page  = params.fetch(:page, 1).to_i
        limit = params.fetch(:limit, DEFAULT_PAGE_SIZE).to_i
        users = @repo.all(page: page, limit: limit)
        respond_with(HTTP_OK, users.map { |u| serialize(u) })
      end

      def show(id)
        user = @repo.find(id)
        if user.nil?
          respond_with(HTTP_NOT_FOUND, { error: "not found" })
        else
          respond_with(HTTP_OK, serialize(user))
        end
      end

      def create(params)
        attrs = filter_params(params)
        user  = @repo.create(attrs)
        respond_with(HTTP_OK, serialize(user))
      end

      def update(id, params)
        user = @repo.find(id)
        return respond_with(HTTP_NOT_FOUND, {}) unless user

        user.update(filter_params(params))
        respond_with(HTTP_OK, serialize(user))
      end

      def destroy(id)
        @repo.delete(id)
        respond_with(HTTP_OK, {})
      end

      private

      def serialize(user)
        { id: user.id, name: user.name, email: user.email }
      end

      def filter_params(raw)
        raw.select { |k, _| ALLOWED_PARAMS.include?(k.to_s) }
      end

      def respond_with(status, body)
        { status: status, body: body }
      end
    end
  end
end

module Api
  module V1
    class SessionsController
      def create(credentials)
        token = authenticate(credentials[:email], credentials[:password])
        respond_with(HTTP_OK, { token: token })
      end

      def destroy(token)
        revoke(token)
        respond_with(HTTP_OK, {})
      end

      private

      def authenticate(email, password)
        "fake-token-#{email}"
      end

      def revoke(token)
        # nothing
      end

      def respond_with(status, body)
        { status: status, body: body }
      end
    end
  end
end

def health_check
  { status: "ok" }
end
