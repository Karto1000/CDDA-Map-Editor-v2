import React from "react";
import GenericWindow from "../generic-window.js";
import MultiMenu from "../../components/multimenu.js";

function Main() {
    return (
        <GenericWindow title={"Open Map"}>
            <MultiMenu tabs={[
                {
                    name: "New Map Editor",
                    content: <></>,
                    isDisabled: true
                },
                {
                    name: "New Map Viewer",
                    content: <></>
                }
            ]}/>
        </GenericWindow>
    );
}

export default Main;
