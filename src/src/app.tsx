import React from 'react';
import {Header} from "./components/header.tsx";
import {useTheme} from "./hooks/useTheme.tsx";
import Main from "./main.tsx";

function App() {
    const [theme, _] = useTheme();

    return (
        <div className={`app ${theme}-theme`}>
            <header>
            </header>
            <Header/>
            <Main/>
        </div>
    );
}

export default App;
