document.addEventListener("DOMContentLoaded", () => {
    const rightLinks = document.getElementById("right-links");

    const jwt = localStorage.getItem("jwt");
    const username = localStorage.getItem("username");
    const userId = localStorage.getItem("user_id");

    if (jwt) {
        rightLinks.innerHTML = "";

        if (username) {
            const usernameElement = document.createElement("a");
            usernameElement.textContent = username;
            usernameElement.href = `/users/${userId}`;

            if (window.location.pathname.split("/")[1] === "users") {
                usernameElement.classList.add("active");
            }

            rightLinks.appendChild(usernameElement);
        }
        const logoutLink = document.createElement("a");
        logoutLink.href = "#";
        logoutLink.id = "logout";
        logoutLink.textContent = "Выйти";
        rightLinks.appendChild(logoutLink);

        logoutLink.addEventListener("click", (event) => {
            event.preventDefault();
            localStorage.removeItem("jwt");
            localStorage.removeItem("username");
            localStorage.removeItem("user_id");

            window.location.href = "/";
        });
    }
});

function renderPost(userId, post) {
    const postElement = document.createElement("div");
    postElement.classList.add("post");

    const postTitleLink = document.createElement("a");
    postTitleLink.href = `/posts/${post.post_id}`;
    postTitleLink.textContent = post.title;

    const postTitle = document.createElement("h2");
    postTitle.appendChild(postTitleLink);

    const postContent = document.createElement("p");
    postContent.textContent = post.content;

    const postMetadata = document.createElement("div");
    postMetadata.classList.add("post-metadata");
    postMetadata.innerHTML = `
            <span>Автор: <a href="/users/${post.user_id}">${post.username}</a></span>
            <span>${formatDateGMT3(new Date(post.created_at))}</span>
        `;

    const likesSection = document.createElement("div");
    likesSection.classList.add("likes-section");

    const likeButton = document.createElement("button");
    likeButton.textContent = "Поставить 👍";
    likeButton.classList.add("like-button");

    const likeCount = document.createElement("span");
    likeCount.textContent = `${post.likes_count} 👍`;
    likeCount.classList.add("like-count");

    likesSection.appendChild(likeButton);
    likesSection.appendChild(likeCount);

    postElement.appendChild(postTitle);
    postElement.appendChild(postContent);
    postElement.appendChild(postMetadata);
    postElement.appendChild(likesSection);

    likeButton.addEventListener("click", () => {
        toggleLike(post.post_id, likeButton, likeCount);
    });

    if (userId === post.user_id.toString()) {
        const deleteButton = document.createElement("button");
        deleteButton.textContent = "Удалить";
        deleteButton.classList.add("delete-button");
        deleteButton.addEventListener("click", () => {
            deletePost(post.post_id, postElement);
        });
        postElement.appendChild(deleteButton);
    }

    return postElement;
}

function toggleLike(postId, likeButton, likeCount) {
    const jwt = localStorage.getItem("jwt");

    fetch(`/api/posts/${postId}/likes`, {
        method: "POST",
        headers: {
            "Authorization": `Bearer ${jwt}`,
            "Content-Type": "application/json",
        },
    })
        .then((response) => response.json())
        .then((data) => {
            if (data.result === "ok") {
                likeButton.textContent = data.like === "Added" ? "Убрать 👍" : "Поставить 👍";
                likeCount.textContent = `${data.likes_count} 👍`;
            } else {
                throw new Error(data.message);
            }
        })
        .catch((error) => {
            console.error("Error toggling like: ", error);
            alert(error.message);
        });
}

function deletePost(postId, postElement) {
    const jwt = localStorage.getItem("jwt");

    fetch(`/api/posts/${postId}`, {
        method: "DELETE",
        headers: {
            "Authorization": `Bearer ${jwt}`,
            "Content-Type": "application/json",
        },
    })
        .then((response) => response.json())
        .then((data) => {
            if (data.result === "ok") {
                postElement.remove();
            } else {
                throw new Error(data.message);
            }
        })
        .catch((error) => {
            console.error("Error deleting post: ", error);
            alert(error.message);
        });
}

function formatDateGMT3(date) {
    const gmt3Date = new Date(date.getTime() + 3 * 60 * 60 * 1000);

    const year = gmt3Date.getFullYear();
    const month = String(gmt3Date.getMonth() + 1).padStart(2, '0');
    const day = String(gmt3Date.getDate()).padStart(2, '0');
    const hours = String(gmt3Date.getHours()).padStart(2, '0');
    const minutes = String(gmt3Date.getMinutes()).padStart(2, '0');
    const seconds = String(gmt3Date.getSeconds()).padStart(2, '0');
    const ampm = hours >= 12 ? 'PM' : 'AM';
    const formattedHours = String(hours % 12 || 12).padStart(2, '0');

    return `${year}/${month}/${day}, ${formattedHours}:${minutes}:${seconds} ${ampm}`;
}
