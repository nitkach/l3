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

---

# Unprotected endpoints
## Index page
Request: `GET "/"`

Response: HTML index page

## Registration page

Request: `GET "/register"`

Response: HTML registration page

## Registration

Request: `POST "/api/register"`

- Require JSON:
```
{
    "username": string,
    "password": string
}
```
Response:
```
{
    "result": "ok",
    "message": "Успешная регистрация!"
}

OR

{
    "result": "err",
    "message": string
}
```

## Login page

Request: `GET "/login"`

Response: HTML login page

## Login

Request: `POST "/api/login"`

- Require JSON:
```
{
    "username": string,
    "password": string
}
```
Response:
```
{
    "result": "ok",
    "jwt": {
        "token": string,
        "username": string,
        "user_id": number
    }
}

OR

{
    "result": "err",
    "message": string
}
```

## Posts page

Request: `GET "/posts"`

Response: HTML posts page

## Post page

Request: `GET "/posts/{post_id}"`

Response: HTML post page

## User posts page

Request: `GET "/users/{user_id}"`

Response: HTML user posts page

# Protected endpoints

## Retrive post content
`GET "/api/posts/{post_id}"`

- Require `post_id` in URL path,
- Require header: `"Authorization": "Bearer {jwt token}"`,

Response:
```
{
    "result": "ok",
    "post": {
        "post_id": number,
        "user_id": number,
        "username": string,
        "title": string,
        "content": string,
        "created_at": string,
        "likes_count": number,
    }
}

OR

{
    "result": "err",
    "message": string
}
```

## Delete post

`DELETE "/api/posts/{post_id}"`

- Require `post_id` in URL path,
- Require header: `"Authorization": "Bearer {jwt token}"`,

Response:
```
{
    "result": "ok",
    "message": "Post deleted successfully."
}

OR

{
    "result": "err",
    "message": "The requested post does not exist." | "You do not have permission to delete this post." | string
}
```

## List all posts

`GET "/api/posts"`

- Require header: `"Authorization": "Bearer {jwt token}"`,

Response:
```
{
    "result": "ok",
    "posts": [
        {
            "post_id": number,
            "user_id": number,
            "username": string,
            "title": string,
            "content": string,
            "created_at": string,
            "likes_count": number,
        },
        ...
        {
            "post_id": number,
            "user_id": number,
            "username": string,
            "title": string,
            "content": string,
            "created_at": string,
            "likes_count": number,
        }
    ]
}

OR

{
    "result": "err",
    "message": string
}
```

## Create post
`POST "/api/posts"`: Creates post
- Require header: `"Authorization": "Bearer {jwt token}"`,
- Require JSON:
```
{
    "title": string,
    "content": string
}
```
Response:
```
{
    "result": "ok",
    "post_id": number
}

OR

{
    "result": "err",
    "message": string
}
```

## Like post

`POST "/api/posts/{post_id}/likes"`: Add or remove like from post by certain user

- Require header: `"Authorization": "Bearer {jwt token}"`,
- Require post_id in path,

Response:
```
{
    "result": "ok",
    "like": "Added" | "Removed",
    "likes_count": number
}

OR

{
    "result": "err",
    "message": string
}
```

## User posts

Request: `GET "/api//users/{user_id}"`

- Require header: `"Authorization": "Bearer {jwt token}"`,
- Require user_id in path,

Response:
```
{
    "result": "ok",
    "username": string,
    "posts": [
        {
            "post_id": number,
            "user_id": number,
            "username": string,
            "title": string,
            "content": string,
            "created_at": string,
            "likes_count": number,
        },
        ...
        {
            "post_id": number,
            "user_id": number,
            "username": string,
            "title": string,
            "content": string,
            "created_at": string,
            "likes_count": number,
        }
    ]
}

OR

{
    "result": "err",
    "message": string
}
```
