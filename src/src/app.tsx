import React, {createContext, useEffect, useRef} from 'react';
import {Panel, PanelGroup, PanelResizeHandle} from "react-resizable-panels";

import "./app.scss"
import {TauriCommand, TauriEvent, ToastType} from "./tauri/events/types.js";
import {TabTypeKind, useTabs, UseTabsReturn} from './shared/hooks/useTabs.ts';
import {WelcomeScreen} from "./shared/components/mainScreens/welcomeScreen.js";
import {NoTabScreen} from "./shared/components/mainScreens/noTabScreen.js";
import {Header} from "./shared/components/header.js";
import MultiMenu from "./shared/components/multimenu.js";
import {getColorFromTheme, Theme, useTheme} from "./shared/hooks/useTheme.js";
import {EditorData} from "./tauri/types/editor.js";
import {useEditorData} from "./shared/hooks/useEditorData.js";
import {MainCanvas} from "./shared/components/mainCanvas.js";
import {useWindows} from "./shared/hooks/useWindows.js";
import {tauriBridge} from "./tauri/events/tauriBridge.js";
import {useThreeSetup} from "./features/three/hooks/useThreeSetup.js";
import {MapViewer} from "./features/viewer/components/mapViewer.js";
import {useTileset} from "./features/sprites/hooks/useTileset.js";
import toast, {ToastBar, Toaster} from "react-hot-toast";
import Icon, {IconName} from "./shared/components/icon.js";
import {useTauriEvent} from "./shared/hooks/useTauriEvent.js";

export const ThemeContext = createContext<{ theme: Theme }>({
    theme: Theme.Dark,
});

export const TabContext = createContext<UseTabsReturn>(null)
export const EditorDataContext = createContext<EditorData>(null)

function App() {
    const eventBus = useRef<EventTarget>(new EventTarget())
    const canvasContainerRef = useRef<HTMLDivElement>()
    const canvasRef = useRef<HTMLCanvasElement>();
    const {threeConfigRef, onResize} = useThreeSetup(
        canvasRef,
        canvasContainerRef
    )

    const [theme] = useTheme(eventBus);
    const editorData = useEditorData(eventBus)
    const tabs = useTabs(eventBus)
    const {spritesheetConfig, tilesheets} = useTileset(eventBus)
    const {openMapWindowRef, settingsWindowRef} = useWindows()

    useEffect(() => {
        (async () => {
            await tauriBridge.invoke(TauriCommand.FRONTEND_READY, {})
        })()
    }, []);

    useTauriEvent(
        TauriEvent.EMIT_TOAST_MESSAGE,
        (d) => {
            if (d.type === ToastType.Error) {
                toast.error(d.message)
            }

            if (d.type === ToastType.Success) {
                toast.success(d.message)
            }
        },
        []
    )

    function getMainBasedOnTab(): React.JSX.Element {
        if (tabs.openedTab !== null) {
            if (tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.Welcome)
                return <WelcomeScreen eventBus={eventBus}/>

            if (tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.MapEditor ||
                tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.LiveViewer)
                return <></>
        }

        return <NoTabScreen setIsCreatingMapWindowOpen={() => {
        }}/>
    }

    return (
        <div className={`app ${theme}-theme`}>
            <Toaster
                position={"bottom-right"}
                toastOptions={{
                    style: {
                        borderRadius: 0,
                        maxWidth: "100%",
                    },
                    success: {
                        icon: <Icon name={IconName.CheckmarkMedium}/>,
                        style: {
                            background: getColorFromTheme(theme, "darkBlue"),
                            border: `2px solid ${getColorFromTheme(theme, "selected")}`,
                        }
                    },
                    error: {
                        icon: <Icon name={IconName.CloseSmall}/>,
                        duration: 5000,
                        style: {
                            background: getColorFromTheme(theme, "delete"),
                            border: `2px solid ${getColorFromTheme(theme, "lightDelete")}`,
                        }
                    }
                }}>
                {(t) => <ToastBar toast={t}/>}
            </Toaster>
            <EditorDataContext.Provider value={editorData}>
                <ThemeContext.Provider value={{theme}}>
                    <TabContext.Provider value={tabs}>
                        <Header
                            openMapWindowRef={openMapWindowRef}
                            settingsWindowRef={settingsWindowRef}
                            eventBus={eventBus}
                        />

                        <PanelGroup direction={'horizontal'}>
                            <Panel collapsible={true} minSize={10} defaultSize={20} maxSize={50}
                                   onResize={onResize}>
                                <div className={"side-panel"}>
                                    <div className={"side-panel-left"}>
                                        {
                                            tabs.shouldDisplayCanvas() ?
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
                                                            content: <></>,
                                                        },
                                                        {
                                                            name: "Monsters",
                                                            content: <></>,
                                                        },
                                                        {
                                                            name: "Signs",
                                                            content: <></>,
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
                            <Panel>
                                <MapViewer
                                    threeConfig={threeConfigRef}
                                    eventBus={eventBus}
                                    spritesheetConfig={spritesheetConfig}
                                    tileInfo={spritesheetConfig.current?.tile_info[0]}
                                    isOpen={tabs.getCurrentTab()?.tab_type === TabTypeKind.LiveViewer}
                                    tilesheets={tilesheets}
                                    canvas={{
                                        canvasRef: canvasRef,
                                        canvasContainerRef: canvasContainerRef
                                    }}
                                />
                                <MainCanvas
                                    canvasRef={canvasRef}
                                    canvasContainerRef={canvasContainerRef}
                                    displayState={tabs.shouldDisplayCanvas() ? "flex" : "none"}
                                />
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
