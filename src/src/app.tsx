import React, {createContext, useEffect, useRef, useState} from 'react';

import "./app.scss"
import {TauriCommand, TauriEvent, ToastType} from "./tauri/events/types.js";
import {TabTypeKind, useTabs, UseTabsReturn} from './shared/hooks/useTabs.ts';
import {WelcomeScreen} from "./shared/components/mainScreens/welcomeScreen.js";
import {NoTabScreen} from "./shared/components/mainScreens/noTabScreen.js";
import {Header} from "./shared/components/header.js";
import {getColorFromTheme, Theme, useTheme} from "./shared/hooks/useTheme.js";
import {EditorData} from "./tauri/types/editor.js";
import {useEditorData} from "./shared/hooks/useEditorData.js";
import {MainCanvas} from "./shared/components/mainCanvas.js";
import {useWindows} from "./shared/hooks/useWindows.js";
import {tauriBridge} from "./tauri/events/tauriBridge.js";
import {useThreeSetup} from "./features/three/hooks/useThreeSetup.js";
import {useTileset} from "./features/sprites/hooks/useTileset.js";
import toast, {ToastBar, Toaster} from "react-hot-toast";
import Icon, {IconName} from "./shared/components/icon.js";
import {useTauriEvent} from "./shared/hooks/useTauriEvent.js";
import {Panel, PanelGroup, PanelResizer} from "@window-splitter/react";
import {clsx} from "clsx";
import {MapViewer} from "./features/viewer/mapViewer.js";
import {SideMenu, SideMenuRef} from "./shared/components/imguilike/sideMenu.js";

export const ThemeContext = createContext<{ theme: Theme }>({
    theme: Theme.Dark,
});

export type SidebarContent = {
    chosenProperties: React.JSX.Element,
    calculatedParameters: React.JSX.Element,
}

export const TabContext = createContext<UseTabsReturn | null>(null)
export const EditorDataContext = createContext<EditorData | null>(null)

function App() {
    const eventBus = useRef<EventTarget>(new EventTarget())
    const canvasContainerRef = useRef<HTMLDivElement>(null)
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const {threeConfigRef, onResize} = useThreeSetup(
        canvasRef,
        canvasContainerRef
    )

    const [theme] = useTheme(eventBus);
    const editorData = useEditorData(eventBus)
    const tabs = useTabs(eventBus)
    const {spritesheetConfig, tilesheets} = useTileset(eventBus, threeConfigRef)
    const {importMapWindowRef, settingsWindowRef, newMapWindowRef, aboutWindowRef} = useWindows()
    const [isGridShowing, setIsGridShowing] = useState<boolean>(true)
    const sideMenuRef = useRef<SideMenuRef>(null)

    const [isSidebarOpen, setIsSidebarOpen] = useState<boolean>(false)

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

            if (tabs.tabs[tabs.openedTab].tab_type === TabTypeKind.LiveViewer)
                return <MapViewer
                    eventBus={eventBus}
                    sideMenuRef={sideMenuRef}
                    tilesheets={tilesheets}
                    tileInfo={spritesheetConfig.current.tile_info[0]}
                    threeConfig={threeConfigRef}
                    canvas={{
                        canvasRef,
                        canvasContainerRef
                    }}
                />
        }

        return <NoTabScreen openMapWindowRef={importMapWindowRef} newMapWindowRef={newMapWindowRef}/>
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
                {(t: any) => <ToastBar toast={t}/>}
            </Toaster>
            <EditorDataContext.Provider value={editorData}>
                <ThemeContext.Provider value={{theme}}>
                    <TabContext.Provider value={tabs}>
                        <Header
                            importMapWindowRef={importMapWindowRef}
                            settingsWindowRef={settingsWindowRef}
                            eventBus={eventBus}
                            newMapWindowRef={newMapWindowRef}
                            showGrid={isGridShowing}
                            setShowGrid={setIsGridShowing}
                            aboutWindowRef={aboutWindowRef}
                        />

                        <PanelGroup>
                            <Panel
                                collapsible
                                collapsed={!isSidebarOpen}
                                onCollapseChange={collapsed => collapsed ? sideMenuRef.current?.collapse() : sideMenuRef.current?.expand()}
                                collapsedSize={"32px"}
                                min={"100px"}
                                max={"1000px"}
                                onResize={onResize}>
                                <div className={clsx("side-panel", isSidebarOpen && "collapsed")}>
                                    {
                                        <SideMenu ref={sideMenuRef} onStateChange={state => setIsSidebarOpen(state)}/>
                                    }
                                </div>
                            </Panel>
                            <PanelResizer className={clsx("resize-handle")} disabled={!isSidebarOpen}
                                          size={"5px"}/>
                            <Panel>
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
