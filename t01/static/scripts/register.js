function register() {
    const registerForm = document.getElementById("register-form");
    if (!(registerForm instanceof HTMLFormElement)) {
        console.error("Cannot find registration form");
        return;
    }
    const registerFormParentNode = registerForm.parentNode;
    if (!registerFormParentNode) {
        console.error("Cannot find registration form parent node");
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

    registerForm.addEventListener("submit", async (event) => {
        event.preventDefault();
        const username = usernameElement.value;
        const password = passwordElement.value;

        // clear any existing messages
        const existingMessages = registerFormParentNode.querySelectorAll(".message");
        existingMessages.forEach(message => message.remove());

        try {
            const response = await fetch("/api/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ username: username, password: password }),
            });

            const data = await response.json();

            if (data.result === "ok") {
                const successMessage = document.createElement("div");
                successMessage.className = "message success-message";
                successMessage.textContent = data.message;
                registerFormParentNode.insertBefore(successMessage, registerForm);

                usernameElement.value = "";
                passwordElement.value = "";
            } else {
                const errorMessage = document.createElement("div");
                errorMessage.className = "message error-message";
                errorMessage.textContent = data.message;
                registerFormParentNode.insertBefore(errorMessage, registerForm);
            }
        } catch (error) {
            console.error("Error during registration:", error);
            alert("Ошибка при регистрации. Пожалуйста, попробуйте позже.");
        }
    });
}
