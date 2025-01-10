## Task

It is necessary to develop a simplified mini-social network where users can register, log in, post messages and like.

The project should include the following functions:

- User registration and authorization: users can create accounts and log in
- Post messages: users can create and delete their messages
- Likes: users can like messages

## Technical requirements

- Use the Axum framework to create a web service.
- Use PostgreSQL and `tokio-postgres` library for working with data
- Use JWT for authentication and authorization
- Use Tokio and async/await for asynchronous execution of tasks
- Provide data validation and error handling

## API structure

- `POST /register`: register a new user
- `POST /login`: log in a user
- `POST /posts`: create a new post
- `GET /posts/{post_id}`: get a post
- `DELETE /posts/{post_id}`: delete a post
- `POST /posts/{post_id}/likes`: like a post.
