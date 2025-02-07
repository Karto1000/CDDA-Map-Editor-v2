import React from "react";
import "./welcomeScreen.scss"
import Icon, {IconName} from "../components/icon.tsx";

export function WelcomeScreen() {
    return (
        <main id={"welcome-main"}>
            <h1>Welcome to the CDDA Map Editor!</h1>
            <img src={"mockup.png"} id={"mockup-image"} alt={"Mockup of the editor"}/>
            <p>This application is still in development and is expected to still contain bugs that the developer hasn't
                bothered to fix yet.</p>
            <p>To get started, click on the <span><Icon name={IconName.AddSmall}/></span> Icon next to the "Welcome to
                the CDDA Map
                Editor" Tab</p>
        </main>
    )
}