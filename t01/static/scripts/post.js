document.addEventListener("DOMContentLoaded", function () {
    const postId = window.location.pathname.split("/").pop();

    const jwt = localStorage.getItem("jwt");

    if (!jwt) {
        window.location.href = "/login";
        return;
    }

    const postContainer = document.querySelector(".post-container");

    fetch(`/api/posts/${postId}`, {
        headers: {
            "Authorization": `Bearer ${jwt}`
        }
    })
        .then(response => {
            if (!response.ok) {
                return response.json().then(err => {
                    throw new Error(err.message);
                });
            }
            return response.json();
        })
        .then(data => {
            if (data.result === "ok") {
                const userId = localStorage.getItem("user_id");

                const postElement = renderPost(userId, data.post);

                postContainer.appendChild(postElement);
            } else {
                throw new Error(data.message);
            }
        })
        .catch(error => {
            console.error("Error:", error);
            postContainer.innerHTML = `<p class="error">${error.message}</p>`;
        });
});
