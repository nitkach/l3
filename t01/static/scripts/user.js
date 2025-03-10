document.addEventListener("DOMContentLoaded", (event) => {
    const userContainer = document.getElementById("user-container");

    const jwt = localStorage.getItem("jwt");
    const userId = window.location.pathname.split("/").pop();

    fetch(`/api/users/${userId}`, {
        headers: {
            "Authorization": `Bearer ${jwt}`
        }
    }).then((response) => response.json())
        .then((data) => {
            if (data.result === "err") {
                userContainer.innerHTML = `<p class="error">${data.message}</p>`;
            } else if (data.result === "ok") {
                renderUserPosts(data.username, data.posts);
            }
        }).catch((error) => {
            console.error("Error fetching user posts: ", error);
            alert("Ошибка при попытке получить посты пользователя. Пожалуйста, попробуйте позже.");
        });
});


function renderUserPosts(username, posts) {
    const userPostsList = document.getElementById("user-posts-list");

    if (posts.length === 0) {
        userPostsList.innerHTML = "<p class=\"message\">Пока что постов нет.</p>";
        return;
    }

    const userId = localStorage.getItem("user_id");
    userPostsList.innerHTML = "";

    const usernameElement = document.createElement("div");
    usernameElement.innerHTML = `<h2>Посты пользователя ${username}</h2>`;
    usernameElement.className = "username-container";
    userPostsList.appendChild(usernameElement);

    posts.forEach((post) => {
        const postElement = renderPost(userId, post);
        userPostsList.appendChild(postElement);
    });
}
