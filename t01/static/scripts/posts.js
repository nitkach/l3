document.addEventListener("DOMContentLoaded", () => {
    const postsContainer = document.getElementById("posts-container");
    const jwt = localStorage.getItem("jwt");

    if (!jwt) {
        postsContainer.innerHTML = "<p class='message'>Вам необходимо <a href='/login'>авторизоваться</a> чтобы просматривать и создавать посты.</p>";
        return;
    }

    fetchPosts();

    const postForm = document.getElementById("post-form");
    postForm.addEventListener("submit", (event) => {
        event.preventDefault();

        const title = document.getElementById("title").value;
        const content = document.getElementById("content").value;

        createPost(title, content);
    });
});

function fetchPosts() {
    const jwt = localStorage.getItem("jwt");
    const postsList = document.getElementById("posts-list");
    const createPostForm = document.getElementById("create-post-form");

    fetch("/api/posts", {
        method: "GET",
        headers: {
            "Authorization": `Bearer ${jwt}`,
            "Content-Type": "application/json",
        },
    })
        .then((response) => response.json())
        .then((data) => {
            if (data.result === "err") {
                postsList.innerHTML = `<p class="error">${data.message}</p>`;
            } else if (data.result === "ok") {
                createPostForm.style.display = "block";
                renderPosts(data.posts);
            }
        })
        .catch((error) => {
            console.error("Error fetching posts:", error);
            postsList.innerHTML = "<p class=\"error\">Произошла ошибка при загрузке постов. Пожалуйста, попробуйте позже.</p>";
        });
}

function createPost(title, content) {
    const jwt = localStorage.getItem("jwt");

    fetch("/api/posts", {
        method: "POST",
        headers: {
            "Authorization": `Bearer ${jwt}`,
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ title, content }),
    })
        .then((response) => response.json())
        .then((data) => {
            if (data.result === "err") {
                alert(`Error: ${data.message}`);
            } else if (data.result === "ok") {
                document.getElementById("post-form").reset();
                fetchPosts();
            }
        })
        .catch((error) => {
            console.error("Error creating post:", error);
            alert("Произошла ошибка при создании поста. Пожалуйста, попробуйте позже.");
        });
}

function renderPosts(posts) {
    const postsList = document.getElementById("posts-list");

    if (posts.length === 0) {
        postsList.innerHTML = "<p class=\"message\">Пока что нет ни одного опубликованного поста.</p>";
        return;
    }

    const userId = localStorage.getItem("user_id");
    postsList.innerHTML = "";
    posts.forEach((post) => {
        const postElement = renderPost(userId, post);
        postsList.appendChild(postElement);
    });
}
