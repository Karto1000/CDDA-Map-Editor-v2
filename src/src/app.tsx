import React from 'react';
import {Header} from "./components/header.tsx";
import {useTheme} from "./hooks/useTheme.tsx";

function App() {
    const [theme, _] = useTheme();

    return (
        <div className={`${theme}-theme`}>
            <header>
            </header>
            <Header/>
        </div>
    );
}

export default App;
