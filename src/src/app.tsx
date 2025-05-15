import React, {createContext, SetStateAction, useEffect, useRef, useState} from 'react';
import {Header} from "./components/header.tsx";
import {Theme, useTheme} from "./hooks/useTheme.ts";
import {invoke} from "@tauri-apps/api/core";
import {TabTypeKind, useTabs, UseTabsReturn} from "./hooks/useTabs.ts";
import {listen} from "@tauri-apps/api/event";
import {invokeTauri, makeCancelable} from "./lib/index.ts";
import {Scene} from "three";
import {useViewer} from "./hooks/useViewer.tsx";
import {useTileset} from "./hooks/useTileset.ts";
import {EditorData, EditorDataRecvEvent, ProjectTypeKind} from "./lib/editor_data.ts";
import {MapDataEvent, MapDataSendCommand} from "./lib/map_data.ts";
import {Panel, PanelGroup, PanelResizeHandle} from "react-resizable-panels";

import "./app.scss"
import MultiMenu from "./components/multimenu.tsx";
import {Fieldset} from "./components/fieldset.tsx";
import {WebviewWindow} from "@tauri-apps/api/webviewWindow";
import {Webview} from "@tauri-apps/api/webview";
import {getCurrentWindow} from "@tauri-apps/api/window";
import settingsWindow from "./windows/settings/main.js";
import {NoTabScreen} from "./components/mainScreens/noTabScreen.js";
import {WelcomeScreen} from "./components/mainScreens/welcomeScreen.js";

export const ThemeContext = createContext<{ theme: Theme, setTheme: React.Dispatch<SetStateAction<Theme>> }>({
    theme: Theme.Dark,
    setTheme: () => {
    }
});

export const TabContext = createContext<UseTabsReturn>(null)
export const EditorDataContext = createContext<EditorData>(null)

function App() {
    const [theme, setTheme] = useTheme();
    const [editorData, setEditorData] = useState<EditorData>()
    const tabs = useTabs()

    const mapEditorCanvasContainerRef = useRef<HTMLDivElement>()
    const mapEditorCanvasRef = useRef<HTMLCanvasElement>();
    const mapEditorSceneRef = useRef<Scene>(new Scene())

    const [tilesheets, spritesheetConfig, isTilesheetLoaded] = useTileset(editorData, mapEditorSceneRef)
    const isDisplayingMapEditor = tabs.tabs[tabs.openedTab]?.tab_type === TabTypeKind.LiveViewer
    const mapEditorCanvasDisplay = isDisplayingMapEditor ? "flex" : "none"

    // Thanks to the legend at https://stackoverflow.com/questions/77775315/how-to-create-mulitwindows-in-tauri-rust-react-typescript-html-css
    const openMapWindowRef = useRef<Webview>()
    const settingsWindowRef = useRef<Webview>()

    const {resize, displayInLeftPanel} = useViewer({
        canvasRef: mapEditorCanvasRef,
        sceneRef: mapEditorSceneRef,
        canvasContainerRef: mapEditorCanvasContainerRef,
        isDisplaying: isDisplayingMapEditor,
        tilesheetsRef: tilesheets,
        openedTab: tabs.openedTab,
        isTilesheetLoaded,
        theme,
        spritesheetConfig,
    })

    useEffect(() => {
        const openedTab = tabs.openedTab

        if (openedTab === null) return
        if (tabs.tabs[openedTab].tab_type !== TabTypeKind.LiveViewer) return

        let update_live_viewer_listen = makeCancelable(
            listen<unknown>(MapDataEvent.UpdateLiveViewer, d => {
                (async () => {
                    await invoke(MapDataSendCommand.ReloadProject);
                    await invokeTauri<unknown, unknown>(MapDataSendCommand.GetSprites, {name: openedTab});
                })()
            })
        );

        (async () => {
            await invoke(MapDataSendCommand.OpenProject, {name: openedTab});
        })();

        return () => {
            update_live_viewer_listen.cancel()
        }
    }, [tabs.openedTab, tabs.tabs]);

    useEffect(() => {
        let unlistenDataChanged = makeCancelable(listen<EditorData>(
            EditorDataRecvEvent.EditorDataChanged,
            async (e) => {
                console.log("Received editor data changed event: ", e.payload, "")
                setEditorData(e.payload)

                if (!e.payload.config.cdda_path) {
                    tabs.addLocalTab(
                        {
                            name: "Welcome to the CDDA Map Editor",
                            tab_type: TabTypeKind.Welcome,
                        }
                    )

                    tabs.setOpenedTab("Welcome to the CDDA Map Editor")
                }
            }))

        invoke("frontend_ready", {})

        return () => {
            unlistenDataChanged.cancel()
        }

        // Disable the warning since we do not want to re-run this
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    function getMainBasedOnTab(): React.JSX.Element {
        if (tabs.openedTab !== null) {
            if (tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.Welcome)
                return <WelcomeScreen/>

            if (tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.MapEditor ||
                tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.LiveViewer)
                return <></>
        }

        return <NoTabScreen setIsCreatingMapWindowOpen={() => {
        }}/>
    }

    return (
        <div className={`app ${theme}-theme`}>
            <EditorDataContext.Provider value={editorData}>
                <ThemeContext.Provider value={{theme, setTheme}}>
                    <TabContext.Provider value={tabs}>
                        <Header openMapWindowRef={openMapWindowRef} settingsWindowRef={settingsWindowRef}/>

                        <PanelGroup direction={'horizontal'}>
                            <Panel collapsible={true} minSize={10} defaultSize={20} maxSize={50} onResize={resize}>
                                <div className={"side-panel"}>
                                    <div className={"side-panel-left"}>
                                        {
                                            isDisplayingMapEditor ?
                                                <MultiMenu tabs={
                                                    [
                                                        {
                                                            name: "Terrain",
                                                            content: <></>
                                                        },
                                                        {
                                                            name: "Furniture",
                                                            content: <></>
                                                        },
                                                        {
                                                            name: "Items",
                                                            content: displayInLeftPanel.items
                                                        },
                                                        {
                                                            name: "Monsters",
                                                            content: displayInLeftPanel.monsters
                                                        },
                                                        {
                                                            name: "Signs",
                                                            content: displayInLeftPanel.signs
                                                        },
                                                        {
                                                            name: "Computers",
                                                            content: <></>
                                                        },
                                                        {
                                                            name: "Gaspumps",
                                                            content: <></>
                                                        },
                                                        {
                                                            name: "Toilets",
                                                            content: <></>
                                                        }
                                                    ]}
                                                />
                                                :
                                                <div>
                                                    <h1>Hey there!</h1>
                                                    <p>This is where you can see the properties of any tiles you hover
                                                        over</p>
                                                    <p>If you wish to close this panel you can just drag the Line
                                                        between this panel and the main content to the left</p>
                                                </div>
                                        }
                                    </div>
                                    <div className={"side-panel-right"}/>
                                </div>
                            </Panel>
                            <PanelResizeHandle hitAreaMargins={{coarse: 30, fine: 10}}/>
                            <Panel onResize={resize}>
                                <div ref={mapEditorCanvasContainerRef}
                                     style={{width: "100%", height: "100%", display: mapEditorCanvasDisplay}}>
                                    {/* This should always be in the dom because then we only have to load the sprites once */}
                                    <canvas ref={mapEditorCanvasRef} tabIndex={0}/>
                                </div>
                                {getMainBasedOnTab()}
                            </Panel>
                        </PanelGroup>
                    </TabContext.Provider>
                </ThemeContext.Provider>
            </EditorDataContext.Provider>
        </div>
    );
}

export default App;
