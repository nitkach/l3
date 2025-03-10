function login() {
    const loginForm = document.getElementById("login-form");
    if (!(loginForm instanceof HTMLFormElement)) {
        console.error("Cannot find login form");
        return;
    }
    const loginFormParentNode = loginForm.parentNode;
    if (!loginFormParentNode) {
        console.error("Cannot find login form parent node");
        return;
    }
    const usernameElement = document.getElementById("username");
    if (!(usernameElement instanceof HTMLInputElement)) {
        console.error("Cannot find username input");
        return;
    }
    const passwordElement = document.getElementById("password");
    if (!(passwordElement instanceof HTMLInputElement)) {
        console.error("Cannot find password input");
        return;
    }
    loginForm.addEventListener("submit", async (event) => {
        event.preventDefault();
        const username = usernameElement.value;
        const password = passwordElement.value;

        const existingMessages = loginFormParentNode.querySelectorAll(".message");
        existingMessages.forEach(message => message.remove());
        try {
            const response = await fetch("/api/login", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ username: username, password: password }),
            });
            const data = await response.json();
            if (data.result === "ok") {
                localStorage.setItem("jwt", data.jwt.token);
                localStorage.setItem("username", data.jwt.username);
                localStorage.setItem("user_id", data.jwt.user_id);

                const successMessage = document.createElement("div");
                successMessage.className = "message success-message";
                successMessage.textContent = "Успешная авторизация!";
                loginFormParentNode.insertBefore(successMessage, loginForm);

                usernameElement.value = "";
                passwordElement.value = "";
            }
            else {
                const errorMessage = document.createElement("div");
                errorMessage.className = "message error-message";
                errorMessage.textContent = data.message;
                loginFormParentNode.insertBefore(errorMessage, loginForm);
            }
        }
        catch (error) {
            console.error("Error during login:", error);
            alert("Ошибка авторизации. Пожалуйста, попробуйте позже.");
        }
    });
}
